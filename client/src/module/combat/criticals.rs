use crate::mapping::entity::player::LocalPlayer;
use crate::mapping::entity::EntityRef;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::state::client;

const HOP_VELOCITY: f64 = 0.42;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CritState {
    Ready,

    Charging,
}

fn enabled() -> bool {
    client()
        .modules
        .get(ModuleId::Criticals)
        .and_then(|arc| {
            arc.lock()
                .ok()
                .map(|module| module.get_module_data().enabled)
        })
        .unwrap_or(false)
}

pub fn prepare(player: &LocalPlayer) -> anyhow::Result<CritState> {
    if !enabled() {
        return Ok(CritState::Ready);
    }
    if player.on_ground()? {
        let motion = player.get_delta_movement()?;
        player.set_delta_movement(motion.x(), HOP_VELOCITY, motion.z())?;
        return Ok(CritState::Charging);
    }

    if player.get_fall_distance()? > 0.0 {
        Ok(CritState::Ready)
    } else {
        Ok(CritState::Charging)
    }
}

#[derive(Debug)]
pub struct CriticalsModule {
    pub module: ModuleData,
}

impl CriticalsModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Criticals,
                description: "Makes the combat auras land critical hits".to_string(),
                category: ModuleCategory::Combat,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
        }
    }
}

impl Module for CriticalsModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
