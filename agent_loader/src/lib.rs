mod command;
mod jvm;
mod library;
mod logging;
mod platform;
mod server;

use std::sync::atomic::{AtomicBool, Ordering};

use ctor::{ctor, dtor};
use jni::sys::{jint, JNI_OK};
use log::info;

static RUNNING: AtomicBool = AtomicBool::new(true);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub(crate) fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

#[ctor]
fn agent_onload() {
    initialize_agent();
}

fn initialize_agent() {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }
    info!("agent loader initialized");

    platform::install_signal_handlers();
    jvm::start_monitor();
    server::start();
}

#[no_mangle]
pub extern "system" fn Agent_OnAttach(
    _vm: *mut jni::sys::JavaVM,
    _options: *mut std::ffi::c_char,
    _reserved: *mut std::ffi::c_void,
) -> jint {
    initialize_agent();
    JNI_OK
}

#[dtor]
fn agent_onunload() {
    shutdown();
}

pub(crate) fn shutdown() {
    if !RUNNING.swap(false, Ordering::SeqCst) {
        return;
    }
    info!("agent loader shutting down");
    library::shutdown();
}
