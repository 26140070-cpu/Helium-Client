use crate::mapping::{FieldType, MappedObject};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::state::minecraft;
use jni::objects::JValue;

#[derive(Debug)]
pub struct NoJumpDelayModule {
    pub module: ModuleData,
}

impl NoJumpDelayModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::NoJumpDelay,
                description: "Removes jump cooldown".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
        }
    }
}

impl Module for NoJumpDelayModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            let _ = player.set_field("noJumpDelay", FieldType::Int, JValue::Int(0));
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
