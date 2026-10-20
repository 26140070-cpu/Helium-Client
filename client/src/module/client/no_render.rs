use crate::mapping::MappedObject;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::{client, minecraft};

#[derive(Debug)]
pub struct NoRenderModule {
    pub module: ModuleData,
}

impl NoRenderModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::NoRender,
                description: "Hides annoying overlays and camera effects".to_string(),
                category: ModuleCategory::Client,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Toggle {
                        name: "FireOverlay".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "HurtCam".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Nausea".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Blindness".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Pumpkin".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "BossBar".to_string(),
                        value: false,
                    },
                    ModuleSetting::Toggle {
                        name: "Scoreboard".to_string(),
                        value: false,
                    },
                    ModuleSetting::Toggle {
                        name: "TotemAnimation".to_string(),
                        value: false,
                    },
                ],
            },
        }
    }

    pub fn is_on(name: &str) -> bool {
        client()
            .modules
            .get(ModuleId::NoRender)
            .and_then(|arc| {
                let m = arc.lock().ok()?;
                if !m.get_module_data().enabled {
                    return Some(false);
                }
                m.get_module_data()
                    .get_setting(name)
                    .and_then(|s| s.get_toggle_value())
            })
            .unwrap_or(false)
    }
}

impl Module for NoRenderModule {
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
            if Self::is_on("Nausea") || Self::is_on("HurtCam") {
                if let Ok(scale) = options.screen_effect_scale() {
                    let _ = scale.force_double(0.0);
                }
            }
            if Self::is_on("HurtCam") {
                if let Ok(tilt) = options.damage_tilt_strength() {
                    let _ = tilt.force_double(0.0);
                }
            }
            if Self::is_on("Nausea") {
                if let Ok(fov) = options.fov_effect_scale() {
                    let _ = fov.force_double(0.0);
                }
            }
        }
        if let Ok(Some(player)) = minecraft().player() {
            if Self::is_on("FireOverlay") {
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
