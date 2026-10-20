use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};

#[derive(Debug)]
pub struct HoleEspModule {
    pub module: ModuleData,
}

impl HoleEspModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::HoleEsp,
                description: "Highlights safe holes with bedrock or obsidian walls".to_string(),
                category: ModuleCategory::Render,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Color {
                        name: "Color".to_string(),
                        value: [0.2, 0.9, 0.45, 1.0],
                    },
                    ModuleSetting::Slider {
                        name: "Range".to_string(),
                        value: 6.0,
                        min: 3.0,
                        max: 6.0,
                    },
                ],
            },
        }
    }
}

impl Module for HoleEspModule {
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
