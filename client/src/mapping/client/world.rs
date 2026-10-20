use crate::mapping::block_entity::BlockEntity;
use crate::mapping::block_state::BlockState;
use crate::mapping::entity::Entity;
use crate::mapping::java::iterable::Iterable;
use crate::mapping::math::BlockPos;
use crate::mapping::{MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JValue};

#[derive(Debug, MappedObject)]
#[mapped(class = Level)]
pub struct World {
    jni_ref: GlobalRef,
}

impl World {
    pub fn new(jni_ref: GlobalRef) -> World {
        World { jni_ref }
    }

    pub fn get_entities(&self) -> anyhow::Result<Vec<Entity>> {
        self.in_frame(|| {
            let iterable_obj = self.call_method("entitiesForRendering", &[])?.l()?;
            let iterable = Iterable::new(mapping().new_global_ref(iterable_obj)?);
            let iterator = iterable.iterator()?;
            let mut entities = Vec::new();
            while iterator.has_next()? {
                entities.push(Entity::new(iterator.next()?));
            }
            Ok(entities)
        })
    }

    pub fn get_block_state(&self, pos: &BlockPos) -> anyhow::Result<BlockState> {
        self.in_frame(|| {
            let mut env = mapping().get_env()?;
            let pos_obj = pos.to_java(&mut env)?;
            let state = self
                .call_method("getBlockState", &[JValue::Object(&pos_obj)])?
                .l()?;
            Ok(BlockState::new(mapping().new_global_ref(state)?))
        })
    }

    pub fn is_block_air(&self, pos: &BlockPos) -> anyhow::Result<bool> {
        Ok(self.get_block_state(pos)?.is_air()?)
    }

    pub fn is_crystal_base(&self, pos: &BlockPos) -> anyhow::Result<bool> {
        Ok(self.get_block_state(pos)?.is_crystal_base()?)
    }

    pub fn is_obsidian(&self, pos: &BlockPos) -> anyhow::Result<bool> {
        Ok(self.get_block_state(pos)?.is_obsidian()?)
    }

    pub fn is_bedrock(&self, pos: &BlockPos) -> anyhow::Result<bool> {
        Ok(self.get_block_state(pos)?.is_bedrock()?)
    }

    pub fn can_place_crystal(&self, pos: &BlockPos) -> anyhow::Result<bool> {
        if !self.is_crystal_base(pos)? {
            return Ok(false);
        }
        let above = pos.offset(0, 1, 0);
        if !self.is_block_air(&above)? {
            return Ok(false);
        }
        let above2 = pos.offset(0, 2, 0);
        let _ = above2;
        Ok(true)
    }

    pub fn get_chunk(&self, x: i32, z: i32) -> anyhow::Result<Option<LevelChunk>> {
        self.in_frame(|| {
            let chunk = self
                .call_method("getChunk", &[JValue::Int(x), JValue::Int(z)])?
                .l()?;
            if chunk.is_null() {
                return Ok(None);
            }
            Ok(Some(LevelChunk::new(mapping().new_global_ref(chunk)?)))
        })
    }
}

#[derive(Debug, MappedObject)]
#[mapped(class = LevelChunk)]
pub struct LevelChunk {
    jni_ref: GlobalRef,
}

impl LevelChunk {
    pub fn new(jni_ref: GlobalRef) -> LevelChunk {
        LevelChunk { jni_ref }
    }

    pub fn get_block_entities(&self) -> anyhow::Result<Vec<BlockEntity>> {
        self.in_frame(|| {
            let map = self.call_method("getBlockEntities", &[])?.l()?;
            let values = mapping()
                .call_method(MinecraftClassType::Map, &map, "values", &[])?
                .l()?;
            let iterable = Iterable::new(mapping().new_global_ref(values)?);
            let iterator = iterable.iterator()?;
            let mut out = Vec::new();
            while iterator.has_next()? {
                out.push(BlockEntity::new(iterator.next()?));
            }
            Ok(out)
        })
    }
}
