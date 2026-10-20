use crate::mapping::MappedObject;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

#[derive(Debug)]
pub struct CleanEffectsModule {
    pub module: ModuleData,
}

impl CleanEffectsModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::CleanEffects,
                description: "Clears potion particles and status fog".to_string(),
                category: ModuleCategory::Client,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Toggle {
                        name: "Particles".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Fog".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Weather".to_string(),
                        value: false,
                    },
                    ModuleSetting::Toggle {
                        name: "LowFire".to_string(),
                        value: true,
                    },
                ],
            },
        }
    }
}

impl Module for CleanEffectsModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn on_tick(&self) -> anyhow::Result<()> {
        if !self.module.enabled {
            return Ok(());
        }
        if let Ok(options) = minecraft().options() {
            if self
                .module
                .get_setting("Fog")
                .and_then(|s| s.get_toggle_value())
                .unwrap_or(true)
            {
                if let Ok(scale) = options.screen_effect_scale() {
                    let _ = scale.force_double(0.0);
                }
            }
        }
        if self
            .module
            .get_setting("LowFire")
            .and_then(|s| s.get_toggle_value())
            .unwrap_or(true)
        {
            if let Ok(Some(player)) = minecraft().player() {
                let _ = player.set_field(
                    "remainingFireTicks",
                    crate::mapping::FieldType::Int,
                    jni::objects::JValue::Int(0),
                );
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
