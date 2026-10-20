use crate::log_buffer;
use crate::state::mapping;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{jbyteArray, jint, jobject};
use jni::{JNIEnv, NativeMethod};
use std::ffi::c_void;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const STREAM_BYTECODE: &[u8] = include_bytes!("../java/HeliumLogStream.class");
const STREAM_CLASS: &str = "HeliumLogStream";
const MAX_DEFINE_ATTEMPTS: u32 = 20;

struct JvmLogState {
    installed: bool,
    define_attempts: u32,
    class: Option<GlobalRef>,
}

static STATE: Mutex<JvmLogState> = Mutex::new(JvmLogState {
    installed: false,
    define_attempts: 0,
    class: None,
});

static LINE_OUT: Mutex<String> = Mutex::new(String::new());
static LINE_ERR: Mutex<String> = Mutex::new(String::new());
static WHICH: AtomicBool = AtomicBool::new(false);

pub fn ensure_installed() {
    if let Err(e) = ensure_installed_inner() {
        log::debug!("jvm_log: install failed: {e}");
    }
    clear_pending();
}

fn clear_pending() {
    if let Ok(env) = mapping().get_env() {
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
    }
}

fn ensure_installed_inner() -> anyhow::Result<()> {
    {
        let state = STATE.lock().unwrap();
        if state.installed {
            return Ok(());
        }
        if state.define_attempts >= MAX_DEFINE_ATTEMPTS {
            return Ok(());
        }
    }

    let mut env = mapping().get_env()?;
    let loader = mapping()
        .game_class_loader()
        .ok_or_else(|| anyhow::anyhow!("class loader not ready"))?;

    let class = match load_existing(&mut env, &loader) {
        Some(c) => c,
        None => env
            .define_class(STREAM_CLASS, loader.as_obj(), STREAM_BYTECODE)
            .map_err(|e| describe(&mut env, "DefineClass", e))?,
    };

    let methods = [
        NativeMethod {
            name: "write".into(),
            sig: "(I)V".into(),
            fn_ptr: helium_write_int as *mut c_void,
        },
        NativeMethod {
            name: "write".into(),
            sig: "([BII)V".into(),
            fn_ptr: helium_write_bytes as *mut c_void,
        },
    ];
    env.register_native_methods(&class, &methods)
        .map_err(|e| describe(&mut env, "RegisterNatives", e))?;

    let class_g = env.new_global_ref(&class)?;

    install_system_streams(&mut env, &class)?;
    try_hook_log4j(&mut env);

    let mut state = STATE.lock().unwrap();
    state.class = Some(class_g);
    state.installed = true;
    state.define_attempts = 0;
    log_buffer::push("INFO", "JVM stdout/stderr + log4j bridge installed");
    log::info!("jvm_log: System.out/err redirected");
    Ok(())
}

fn load_existing<'a>(env: &mut JNIEnv<'a>, loader: &GlobalRef) -> Option<JClass<'a>> {
    let Ok(name) = env.new_string(STREAM_CLASS) else {
        return None;
    };
    match env
        .call_method(
            loader.as_obj(),
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )
        .and_then(|v| v.l())
    {
        Ok(class) if !class.is_null() => Some(JClass::from(class)),
        _ => {
            let _ = env.exception_clear();
            None
        }
    }
}

fn install_system_streams(env: &mut JNIEnv, stream_class: &JClass) -> anyhow::Result<()> {
    let out_stream = env
        .new_object(stream_class, "(I)V", &[JValue::Int(0)])
        .map_err(|e| describe(env, "new HeliumLogStream out", e))?;
    let err_stream = env
        .new_object(stream_class, "(I)V", &[JValue::Int(1)])
        .map_err(|e| describe(env, "new HeliumLogStream err", e))?;

    let print_stream_class = env.find_class("java/io/PrintStream")?;

    let out_ps = env
        .new_object(
            &print_stream_class,
            "(Ljava/io/OutputStream;Z)V",
            &[JValue::Object(&out_stream), JValue::Bool(1)],
        )
        .map_err(|e| describe(env, "PrintStream out", e))?;

    let err_ps = env
        .new_object(
            &print_stream_class,
            "(Ljava/io/OutputStream;Z)V",
            &[JValue::Object(&err_stream), JValue::Bool(1)],
        )
        .map_err(|e| describe(env, "PrintStream err", e))?;

    let system = env.find_class("java/lang/System")?;
    env.call_static_method(
        &system,
        "setOut",
        "(Ljava/io/PrintStream;)V",
        &[JValue::Object(&out_ps)],
    )
    .map_err(|e| describe(env, "System.setOut", e))?;
    env.call_static_method(
        &system,
        "setErr",
        "(Ljava/io/PrintStream;)V",
        &[JValue::Object(&err_ps)],
    )
    .map_err(|e| describe(env, "System.setErr", e))?;
    Ok(())
}

fn try_hook_log4j(env: &mut JNIEnv) {
    let result = (|| -> anyhow::Result<()> {
        let log_manager = env.find_class("org/apache/logging/log4j/LogManager")?;
        let context = env
            .call_static_method(
                &log_manager,
                "getContext",
                "(Z)Lorg/apache/logging/log4j/spi/LoggerContext;",
                &[JValue::Bool(0)],
            )?
            .l()?;
        if context.is_null() {
            return Ok(());
        }
        log_buffer::push(
            "INFO",
            "log4j LoggerContext detected (stdout mirror active)",
        );
        Ok(())
    })();
    if let Err(e) = result {
        let _ = env.exception_clear();
        log::debug!("jvm_log: log4j not hooked ({e})");
    }
}

fn feed_bytes(is_err: bool, data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let buf = if is_err { &LINE_ERR } else { &LINE_OUT };
    if let Ok(mut line) = buf.lock() {
        for ch in text.chars() {
            if ch == '\n' {
                let msg = line.trim_end_matches('\r').to_string();
                line.clear();
                if !msg.is_empty() {
                    let level = if is_err {
                        if msg.contains("ERROR") || msg.contains("Exception") {
                            "ERROR"
                        } else if msg.contains("WARN") {
                            "WARN"
                        } else {
                            "ERR"
                        }
                    } else if msg.contains("ERROR") {
                        "ERROR"
                    } else if msg.contains("WARN") {
                        "WARN"
                    } else {
                        "GAME"
                    };
                    log_buffer::push(level, msg);
                }
            } else {
                line.push(ch);
                if line.len() > 4000 {
                    let msg = std::mem::take(&mut *line);
                    log_buffer::push(if is_err { "ERR" } else { "GAME" }, msg);
                }
            }
        }
    }
}

extern "system" fn helium_write_int(mut env: JNIEnv, this: jobject, b: jint) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let is_err = read_kind(&mut env, this);
        feed_bytes(is_err, &[b as u8]);
    }));
}

extern "system" fn helium_write_bytes(
    mut env: JNIEnv,
    this: jobject,
    arr: jbyteArray,
    off: jint,
    len: jint,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if arr.is_null() || len <= 0 {
            return;
        }
        let is_err = read_kind(&mut env, this);
        let Ok(bytes) = env.convert_byte_array(unsafe { jni::objects::JByteArray::from_raw(arr) })
        else {
            return;
        };
        let start = off as usize;
        let end = ((off + len) as usize).min(bytes.len());
        if start >= bytes.len() {
            return;
        }
        feed_bytes(is_err, &bytes[start..end]);
    }));
}

fn read_kind(env: &mut JNIEnv, this: jobject) -> bool {
    if this.is_null() {
        return false;
    }
    let obj = unsafe { JObject::from_raw(this) };
    match env.get_field(&obj, "kind", "I") {
        Ok(v) => v.i().unwrap_or(0) != 0,
        Err(_) => false,
    }
}

fn describe(env: &mut JNIEnv, what: &str, error: jni::errors::Error) -> anyhow::Error {
    if env.exception_check().unwrap_or(false) {
        let detail = env
            .exception_occurred()
            .ok()
            .and_then(|throwable| {
                let _ = env.exception_clear();
                env.call_method(&throwable, "toString", "()Ljava/lang/String;", &[])
                    .ok()
            })
            .and_then(|v| v.l().ok())
            .and_then(|obj| {
                let jstr = JString::from(obj);
                env.get_string(&jstr)
                    .ok()
                    .map(|s| s.to_string_lossy().into_owned())
            });
        if let Some(d) = detail {
            return anyhow::anyhow!("{what}: {d}");
        }
    }
    anyhow::anyhow!("{what}: {error}")
}

pub fn tick_install() {
    let attempts = STATE.lock().unwrap().define_attempts;
    if STATE.lock().unwrap().installed {
        return;
    }
    STATE.lock().unwrap().define_attempts = attempts.saturating_add(1);
    ensure_installed();
}
