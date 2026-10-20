use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};

#[derive(Debug)]
pub struct MineralEspModule {
    pub module: ModuleData,
}

impl MineralEspModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::MineralEsp,
                description: "Reveals nearby ores through blocks with an incremental world scan"
                    .to_string(),
                category: ModuleCategory::Render,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Color {
                        name: "Color".to_string(),
                        value: [0.2, 0.8, 1.0, 1.0],
                    },
                    ModuleSetting::Slider {
                        name: "Range".to_string(),
                        value: 12.0,
                        min: 3.0,
                        max: 24.0,
                    },
                    ModuleSetting::Slider {
                        name: "Vertical Range".to_string(),
                        value: 12.0,
                        min: 4.0,
                        max: 24.0,
                    },
                ],
            },
        }
    }
}

impl Module for MineralEspModule {
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
