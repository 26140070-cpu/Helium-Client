use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;
use std::sync::Mutex;

#[derive(Debug)]
pub struct SoundFXModule {
    pub module: ModuleData,
    original_master: Mutex<Option<f64>>,
}

impl SoundFXModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::SoundFX,
                description: "Client sound volume and SFX tweaks".to_string(),
                category: ModuleCategory::Client,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Slider {
                        name: "Master".to_string(),
                        value: 100.0,
                        min: 0.0,
                        max: 100.0,
                    },
                    ModuleSetting::Toggle {
                        name: "MuteExplosion".to_string(),
                        value: false,
                    },
                    ModuleSetting::Toggle {
                        name: "MuteWeather".to_string(),
                        value: false,
                    },
                    ModuleSetting::Toggle {
                        name: "MuteWither".to_string(),
                        value: false,
                    },
                    ModuleSetting::Toggle {
                        name: "HitSounds".to_string(),
                        value: false,
                    },
                    ModuleSetting::Slider {
                        name: "HitVolume".to_string(),
                        value: 80.0,
                        min: 0.0,
                        max: 100.0,
                    },
                ],
            },
            original_master: Mutex::new(None),
        }
    }
}

impl Module for SoundFXModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn on_tick(&self) -> anyhow::Result<()> {
        let _ = minecraft();
        let _ = self
            .module
            .get_setting("Master")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(100.0);
        Ok(())
    }
    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }
    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
