use crate::mapping::entity::EntityRef;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

#[derive(Debug)]
pub struct JesusModule {
    pub module: ModuleData,
}

impl JesusModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Jesus,
                description: "Walk on water and lava".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Slider {
                    name: "Boost".to_string(),
                    value: 0.3,
                    min: 0.05,
                    max: 1.5,
                }],
            },
        }
    }
}

impl Module for JesusModule {
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
        if motion.y() < 0.0 && !player.on_ground()? {
            let boost = self
                .module
                .get_setting("Boost")
                .and_then(|s| s.get_slider_value())
                .unwrap_or(0.3) as f64;
            if player.is_in_water()? || player.is_in_lava()? {
                player.set_delta_movement(motion.x() * 1.05, boost, motion.z() * 1.05)?;
            }
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
