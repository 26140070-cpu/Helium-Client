#![allow(dead_code)]

extern crate jni;
#[cfg(all(target_os = "macos", not(target_arch = "x86_64")))]
compile_error!("the macOS client currently requires an x86_64 JVM (Intel or Rosetta)");

mod config;
mod friends;
mod graphic;
mod jvm_log;
mod log_buffer;
mod mapping;
mod module;
pub mod net;
mod shell;
mod state;

#[allow(clippy::all, clippy::pedantic)]
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::graphic::hook::{install_hooks, uninstall_hooks};
use crate::module::client::CleanEffectsModule;
use crate::module::client::HudEditorModule;
use crate::module::client::NoRenderModule;
use crate::module::client::NotificationsModule;
use crate::module::client::SoundFXModule;
use crate::module::client::TerminalModule;
use crate::module::combat::aimbot::AimbotModule;
use crate::module::combat::autocrystal::AutoCrystalModule;
use crate::module::combat::criticals::CriticalsModule;
use crate::module::combat::killaura::KillAuraModule;
use crate::module::combat::mobaura::MobAuraModule;
use crate::module::combat::reach::ReachModule;
use crate::module::combat::triggerbot::TriggerBotModule;
use crate::module::combat::velocity::VelocityModule;
use crate::module::misc::antiafk::AntiAfkModule;
use crate::module::movement::antivoid::AntiVoidModule;
use crate::module::movement::autowalk::AutoWalkModule;
use crate::module::movement::blink::BlinkModule;
use crate::module::movement::fly::FlyModule;
use crate::module::movement::jesus::JesusModule;
use crate::module::movement::nofall::NoFallModule;
use crate::module::movement::nojumpdelay::NoJumpDelayModule;
use crate::module::movement::noslowdown::NoSlowdownModule;
use crate::module::movement::parkour::ParkourModule;
use crate::module::movement::safewalk::SafeWalkModule;
use crate::module::movement::speed::SpeedModule;
use crate::module::movement::sprint::SprintModule;
use crate::module::movement::step::StepModule;
use crate::module::movement::strafe::StrafeModule;
use crate::module::player::auto_eat::AutoEatModule;
use crate::module::player::autoarmor::AutoArmorModule;
use crate::module::player::autototem::AutoTotemModule;
use crate::module::player::cheststealer::ChestStealerModule;
use crate::module::player::freecam::FreecamModule;
use crate::module::render::chest_esp::ChestEspModule;
use crate::module::render::fullbright::FullbrightModule;
use crate::module::render::hole_esp::HoleEspModule;
use crate::module::render::mineral_esp::MineralEspModule;
use crate::module::render::mob_esp::MobEspModule;
use crate::module::render::nametags::NametagsModule;
use crate::module::render::player_esp::PlayerEspModule;
use crate::module::render::tracers::TracersModule;
use crate::module::world::nuker::NukerModule;
use crate::module::world::scaffold::ScaffoldModule;
use crate::module::world::surround::SurroundModule;
use crate::state::{client, init};
use log::{error, info, LevelFilter};
use simplelog::{CombinedLogger, Config, SharedLogger, WriteLogger};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub static RUNNING: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn initialize_client() {
    if RUNNING.swap(true, Ordering::SeqCst) {
        info!("Client already initialized");
        return;
    }

    let log_path = config::base_dir().join("helium_client.log");
    match CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create(&log_path).unwrap(),
        ),
        Box::new(crate::log_buffer::BufferLogger),
    ]) {
        Ok(_) => {
            crate::log_buffer::push("INFO", "Logger initialized");
            info!("Logger initialized");
        }
        Err(e) => eprintln!("Error during logger initialization: {:?}", e),
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("HeliumClient panicked! Restoring input/render hooks…");
        cleanup_client();
        default_hook(panic_info);
    }));

    thread::spawn(|| {
        info!("Starting HeliumClient…");

        if let Err(e) = init() {
            error!("Client initialization failed: {e}");
            return;
        }
        register_modules();

        config::load();
        if let Err(e) = install_hooks() {
            error!("Failed to install hooks: {e}");
        }
        info!("HeliumClient started");
    });
}

#[no_mangle]
pub extern "C" fn cleanup_client() {
    info!("Client cleanup in progress…");
    RUNNING.store(false, Ordering::SeqCst);
    uninstall_hooks();
    crate::graphic::input::cleanup();

    if let Some(client) = crate::state::try_client() {
        client.modules.set_active(false);
    }

    crate::state::teardown();
    info!("Client cleanup completed");
}

fn register_modules() {
    let modules = &client().modules;
    modules.register(FlyModule::new());
    modules.register(BlinkModule::new());
    modules.register(NoFallModule::new());
    modules.register(KillAuraModule::new());
    modules.register(MobAuraModule::new());
    modules.register(AimbotModule::new());
    modules.register(CriticalsModule::new());
    modules.register(VelocityModule::new());
    modules.register(TriggerBotModule::new());
    modules.register(AutoCrystalModule::new());
    modules.register(HudEditorModule::new());
    modules.register(NoRenderModule::new());
    modules.register(SoundFXModule::new());
    modules.register(NotificationsModule::new());
    modules.register(CleanEffectsModule::new());
    modules.register(TerminalModule::new());
    modules.register(ReachModule::new());
    modules.register(PlayerEspModule::new());
    modules.register(MobEspModule::new());
    modules.register(ChestEspModule::new());
    modules.register(HoleEspModule::new());
    modules.register(MineralEspModule::new());
    modules.register(TracersModule::new());
    modules.register(FullbrightModule::new());
    modules.register(AntiAfkModule::new());
    modules.register(SprintModule::new());
    modules.register(NametagsModule::new());
    modules.register(SpeedModule::new());
    modules.register(NoSlowdownModule::new());
    modules.register(JesusModule::new());
    modules.register(StepModule::new());
    modules.register(SafeWalkModule::new());
    modules.register(AutoWalkModule::new());
    modules.register(AntiVoidModule::new());
    modules.register(StrafeModule::new());
    modules.register(NoJumpDelayModule::new());
    modules.register(ParkourModule::new());
    modules.register(AutoTotemModule::new());
    modules.register(AutoEatModule::new());
    modules.register(ChestStealerModule::new());
    modules.register(AutoArmorModule::new());
    modules.register(NukerModule::new());
    modules.register(ScaffoldModule::new());
    modules.register(SurroundModule::new());
    modules.register(FreecamModule::new());
}
