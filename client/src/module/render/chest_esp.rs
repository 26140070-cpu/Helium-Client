use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};

#[derive(Debug)]
pub struct ChestEspModule {
    pub module: ModuleData,
}

impl ChestEspModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::ChestEsp,
                description: "Draws a 3D box around containers".to_string(),
                category: ModuleCategory::Render,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Toggle {
                        name: "Containers".to_string(),
                        value: true,
                    },
                    ModuleSetting::Color {
                        name: "Color".to_string(),
                        value: [1.0, 0.55, 0.12, 1.0],
                    },
                    ModuleSetting::Toggle {
                        name: "Distance".to_string(),
                        value: true,
                    },
                ],
            },
        }
    }
}

impl Module for ChestEspModule {
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
