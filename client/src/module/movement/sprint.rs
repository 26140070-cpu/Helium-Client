use crate::mapping::entity::EntityRef;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

const MOVING_THRESHOLD: f64 = 0.03;

const JUMP_VELOCITY: f64 = 0.42;

const SPRINT_JUMP_BOOST: f64 = 0.2;

#[derive(Debug)]
pub struct SprintModule {
    pub module: ModuleData,
}

impl SprintModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Sprint,
                description: "Sprints automatically while moving".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Toggle {
                    name: "Bhop".to_string(),
                    value: false,
                }],
            },
        }
    }

    fn bhop(&self) -> bool {
        self.module
            .get_setting("Bhop")
            .and_then(|setting| setting.get_toggle_value())
            .unwrap_or(false)
    }
}

impl Module for SprintModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let Some(player) = minecraft().player()? else {
            return Ok(());
        };

        let motion = player.get_delta_movement()?;
        let horizontal = (motion.x() * motion.x() + motion.z() * motion.z()).sqrt();
        if horizontal <= MOVING_THRESHOLD {
            return Ok(());
        }

        if !player.is_sprinting()? {
            player.set_sprinting(true)?;
        }

        if self.bhop() && player.on_ground()? {
            let yaw = player.get_yaw()?.to_radians();
            player.set_delta_movement(
                motion.x() - (yaw.sin() as f64) * SPRINT_JUMP_BOOST,
                JUMP_VELOCITY,
                motion.z() + (yaw.cos() as f64) * SPRINT_JUMP_BOOST,
            )?;
        }

        Ok(())
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
