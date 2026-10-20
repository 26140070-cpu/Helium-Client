use crate::mapping::entity::EntityRef;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

#[derive(Debug)]
pub struct AntiVoidModule {
    pub module: ModuleData,
}

impl AntiVoidModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::AntiVoid,
                description: "Saves you from falling into the void".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Slider {
                    name: "Y".to_string(),
                    value: 5.0,
                    min: -64.0,
                    max: 20.0,
                }],
            },
        }
    }
}

impl Module for AntiVoidModule {
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
        let threshold = self
            .module
            .get_setting("Y")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(5.0) as f64;
        let pos = player.get_position()?;
        if pos.y() < threshold {
            let motion = player.get_delta_movement()?;
            player.set_delta_movement(motion.x(), 1.2, motion.z())?;
            player.reset_fall_distance()?;
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
