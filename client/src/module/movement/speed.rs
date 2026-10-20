use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

#[derive(Debug)]
pub struct SpeedModule {
    pub module: ModuleData,
}

impl SpeedModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Speed,
                description: "Increases movement speed".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Slider {
                        name: "Multiplier".to_string(),
                        value: 1.5,
                        min: 1.0,
                        max: 5.0,
                    },
                    ModuleSetting::Toggle {
                        name: "GroundOnly".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "AutoSprint".to_string(),
                        value: true,
                    },
                ],
            },
        }
    }
}

impl Module for SpeedModule {
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
        let ground_only = self
            .module
            .get_setting("GroundOnly")
            .and_then(|s| s.get_toggle_value())
            .unwrap_or(true);
        if ground_only && !player.on_ground()? {
            return Ok(());
        }
        if !player.has_move_input()? {
            return Ok(());
        }
        let scale = self
            .module
            .get_setting("Multiplier")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(1.5) as f64;
        let auto_sprint = self
            .module
            .get_setting("AutoSprint")
            .and_then(|s| s.get_toggle_value())
            .unwrap_or(true);
        if auto_sprint && !player.is_sprinting()? {
            player.set_sprinting(true)?;
        }
        let motion = player.get_delta_movement()?;
        player.set_delta_movement(motion.x() * scale, motion.y(), motion.z() * scale)?;
        Ok(())
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
