use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};

#[derive(Debug)]
pub struct PlayerEspModule {
    pub module: ModuleData,
}

impl PlayerEspModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::PlayerEsp,
                description: "Draws a 3D box around players".to_string(),
                category: ModuleCategory::Render,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Color {
                        name: "Color".to_string(),
                        value: [1.0, 0.27, 0.27, 1.0],
                    },
                    ModuleSetting::Toggle {
                        name: "Name".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Distance".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Health".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Ignore Friends".to_string(),
                        value: false,
                    },
                    ModuleSetting::Slider {
                        name: "Range".to_string(),
                        value: 64.0,
                        min: 16.0,
                        max: 256.0,
                    },
                ],
            },
        }
    }
}

impl Module for PlayerEspModule {
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
