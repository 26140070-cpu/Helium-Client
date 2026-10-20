use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::client;

#[derive(Debug)]
pub struct HudEditorModule {
    pub module: ModuleData,
}

impl HudEditorModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::HudEditor,
                description: "Configure, preview, and position HUD elements".to_string(),
                category: ModuleCategory::Client,
                key_bind: KeyboardKey::KeyNone,
                enabled: true,
                settings: vec![
                    ModuleSetting::Toggle {
                        name: "Watermark".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Arraylist".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Notifications".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Coordinates".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Speed".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "FPS".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Target HUD".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Direction".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Player Status".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Grid".to_string(),
                        value: true,
                    },
                    ModuleSetting::Slider {
                        name: "Scale".to_string(),
                        value: 1.0,
                        min: 0.7,
                        max: 1.5,
                    },
                    ModuleSetting::Slider {
                        name: "ArrayOpacity".to_string(),
                        value: 165.0,
                        min: 40.0,
                        max: 255.0,
                    },
                    ModuleSetting::Color {
                        name: "Accent".to_string(),
                        value: [0.35, 0.75, 1.0, 1.0],
                    },
                    ModuleSetting::Toggle {
                        name: "Rainbow".to_string(),
                        value: false,
                    },
                ],
            },
        }
    }

    pub fn enabled_setting(name: &str, fallback: bool) -> bool {
        client()
            .modules
            .get(ModuleId::HudEditor)
            .and_then(|arc| {
                let m = arc.lock().ok()?;
                if !m.get_module_data().enabled {
                    return Some(fallback);
                }
                m.get_module_data()
                    .get_setting(name)
                    .and_then(|s| s.get_toggle_value())
            })
            .unwrap_or(fallback)
    }

    pub fn scale() -> f32 {
        client()
            .modules
            .get(ModuleId::HudEditor)
            .and_then(|arc| {
                let m = arc.lock().ok()?;
                m.get_module_data()
                    .get_setting("Scale")
                    .and_then(|s| s.get_slider_value())
            })
            .unwrap_or(1.0)
    }

    pub fn array_opacity() -> u8 {
        client()
            .modules
            .get(ModuleId::HudEditor)
            .and_then(|arc| {
                let m = arc.lock().ok()?;
                m.get_module_data()
                    .get_setting("ArrayOpacity")
                    .and_then(|s| s.get_slider_value())
            })
            .unwrap_or(165.0) as u8
    }

    pub fn rainbow() -> bool {
        Self::enabled_setting("Rainbow", false)
    }

    pub fn accent() -> [f32; 4] {
        client()
            .modules
            .get(ModuleId::HudEditor)
            .and_then(|arc| {
                let m = arc.lock().ok()?;
                match m.get_module_data().get_setting("Accent") {
                    Some(crate::module::ModuleSetting::Color { value, .. }) => Some(*value),
                    _ => None,
                }
            })
            .unwrap_or([0.35, 0.75, 1.0, 1.0])
    }
}

impl Module for HudEditorModule {
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
