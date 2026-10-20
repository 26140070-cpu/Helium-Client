use crate::mapping::entity::EntityRef;
use crate::mapping::{FieldType, MappedObject};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::state::minecraft;
use jni::objects::JValue;

#[derive(Debug)]
pub struct AutoWalkModule {
    pub module: ModuleData,
}

impl AutoWalkModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::AutoWalk,
                description: "Walks forward automatically".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
        }
    }
}

impl Module for AutoWalkModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            let _ = player.set_field("zza", FieldType::Float, JValue::Float(0.0));
        }
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            player.set_field("zza", FieldType::Float, JValue::Float(1.0))?;
            if !player.is_sprinting()? {
                player.set_sprinting(true)?;
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
