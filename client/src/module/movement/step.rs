use crate::mapping::{FieldType, MappedObject};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;
use jni::objects::JValue;

#[derive(Debug)]
pub struct StepModule {
    pub module: ModuleData,
}

impl StepModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Step,
                description: "Step up full blocks".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Slider {
                    name: "Height".to_string(),
                    value: 1.0,
                    min: 0.6,
                    max: 2.5,
                }],
            },
        }
    }

    fn height(&self) -> f32 {
        self.module
            .get_setting("Height")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(1.0)
    }
}

impl Module for StepModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            let _ = player.set_field("maxUpStep", FieldType::Float, JValue::Float(0.6));
        }
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            let _ = player.set_field("maxUpStep", FieldType::Float, JValue::Float(self.height()));
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
