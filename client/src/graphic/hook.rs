use crate::graphic::platform;
use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::{gl, state, RUNNING};
use ilhook::x64::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers};
use log::info;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Mutex, OnceLock};

static LAST_TICK: AtomicI32 = AtomicI32::new(0);
static GL_LOADED: AtomicBool = AtomicBool::new(false);

pub struct HookHandle(HookPoint);

unsafe impl Send for HookHandle {}
unsafe impl Sync for HookHandle {}

static GLOBAL_HOOK: OnceLock<Mutex<Option<HookHandle>>> = OnceLock::new();

fn global_hook() -> &'static Mutex<Option<HookHandle>> {
    GLOBAL_HOOK.get_or_init(|| Mutex::new(None))
}

// --- Per-frame logic -------------------------------------------------------

/// `ilhook` trampoline for the host's buffer-swap function.
unsafe extern "win64" fn swap_buffers_hook(_registers: *mut Registers, _user_data: usize) {
    on_frame();
}

unsafe fn on_frame() {
    if !RUNNING.load(Ordering::SeqCst) {
        return;
    }

    if !GL_LOADED.load(Ordering::Relaxed) {
        gl::load_with(platform::gl_proc_address);
        GL_LOADED.store(true, Ordering::Relaxed);
    }

    check_tick();

    crate::module::combat::rotation::update();
    render_overlay();
}

unsafe fn render_overlay() {
    crate::graphic::input::init();

    crate::graphic::ui_engine::render_egui_ui();
}

fn check_tick() {
    if state::env().is_err() {
        return;
    }

    let mc = state::minecraft();
    let in_world = matches!(mc.world(), Ok(Some(_)));
    let player = match mc.player() {
        Ok(Some(player)) => Some(player),
        Ok(None) | Err(_) => None,
    };
    let alive = player
        .as_ref()
        .is_some_and(|player| player.is_alive().unwrap_or(false));
    state::client().modules.set_active(in_world && alive);
    if !in_world || !alive {
        return;
    }
    let Some(player) = player else {
        return;
    };

    let tick_count = match player.get_tick_count() {
        Ok(count) => count,
        Err(_) => return,
    };

    if tick_count > LAST_TICK.load(Ordering::Relaxed) {
        LAST_TICK.store(tick_count, Ordering::Relaxed);

        crate::net::ensure_installed();
        crate::jvm_log::tick_install();
        state::client().modules.tick();
    }
}

fn store_hook(hook: HookPoint, label: &str) {
    let mut guard = global_hook().lock().unwrap();
    if guard.is_some() {
        info!("Replacing the existing buffer-swap hook.");
    }
    *guard = Some(HookHandle(hook));
    info!(">>> HOOK ACTIVE ON: {} <<<", label);
}

fn hooker_for(target_addr: usize) -> Hooker {
    Hooker::new(
        target_addr,
        HookType::JmpBack(swap_buffers_hook),
        CallbackOption::None,
        0,
        HookFlags::empty(),
    )
}

pub fn install_hooks() -> anyhow::Result<()> {
    for target in platform::frame_hook_targets() {
        let result = unsafe { hooker_for(target.address).hook() };
        match result {
            Ok(hook) => {
                store_hook(hook, &target.label);
                return Ok(());
            }
            Err(e) => info!("hook install failed on {}: {e:?}", target.label),
        }
    }
    Err(anyhow::anyhow!(
        "no buffer-swap target could be hooked on this platform"
    ))
}

pub fn uninstall_hooks() {
    let mut guard = global_hook().lock().unwrap();
    if guard.take().is_some() {
        info!("Buffer-swap hook removed; memory restored.");
    } else {
        info!("No active hook to remove.");
    }
}
