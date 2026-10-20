use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JObject, JValue};

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = BlockState)]
pub struct BlockState {
    jni_ref: GlobalRef,
}

impl BlockState {
    pub fn new(jni_ref: GlobalRef) -> BlockState {
        BlockState { jni_ref }
    }

    pub fn is_air(&self) -> anyhow::Result<bool> {
        Ok(self.call_method("isAir", &[])?.z()?)
    }

    pub fn get_block(&self) -> anyhow::Result<GlobalRef> {
        self.in_frame(|| {
            let block = self.call_method("getBlock", &[])?.l()?;
            mapping().new_global_ref(block)
        })
    }

    pub fn is_of(&self, block_field: &str) -> anyhow::Result<bool> {
        self.in_frame(|| {
            let expected = mapping()
                .get_static_field(
                    MinecraftClassType::Blocks,
                    block_field,
                    FieldType::Object(MinecraftClassType::Block),
                )?
                .l()?;
            let actual = self.call_method("getBlock", &[])?.l()?;
            let mut env = mapping().get_env()?;
            Ok(env.is_same_object(&actual, &expected)?)
        })
    }

    pub fn is_obsidian(&self) -> anyhow::Result<bool> {
        self.is_of("OBSIDIAN")
    }

    pub fn is_bedrock(&self) -> anyhow::Result<bool> {
        self.is_of("BEDROCK")
    }

    pub fn is_crystal_base(&self) -> anyhow::Result<bool> {
        Ok(self.is_obsidian()? || self.is_bedrock()?)
    }

    pub fn is_respawn_anchor(&self) -> anyhow::Result<bool> {
        self.is_of("RESPAWN_ANCHOR")
    }
}

pub fn block_static(name: &str) -> anyhow::Result<JObject<'static>> {
    Ok(mapping()
        .get_static_field(
            MinecraftClassType::Blocks,
            name,
            FieldType::Object(MinecraftClassType::Block),
        )?
        .l()?)
}
