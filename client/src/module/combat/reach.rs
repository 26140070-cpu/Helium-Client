use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};

#[derive(Debug)]
pub struct ReachModule {
    pub module: ModuleData,
}

impl ReachModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Reach,
                description: "Extends attack range used by combat modules".to_string(),
                category: ModuleCategory::Combat,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Slider {
                    name: "Range".to_string(),
                    value: 4.2,
                    min: 3.0,
                    max: 6.0,
                }],
            },
        }
    }

    pub fn range_override() -> Option<f32> {
        use crate::module::ModuleId;
        use crate::state::client;
        client().modules.get(ModuleId::Reach).and_then(|arc| {
            let module = arc.lock().ok()?;
            if !module.get_module_data().enabled {
                return None;
            }
            module
                .get_module_data()
                .get_setting("Range")
                .and_then(|s| s.get_slider_value())
        })
    }
}

impl Module for ReachModule {
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
