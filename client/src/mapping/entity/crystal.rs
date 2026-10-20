use crate::mapping::entity::EntityRef;
use crate::mapping::math::BlockPos;
use crate::mapping::MappedObject;
use jni::objects::GlobalRef;

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = EndCrystal)]
pub struct EndCrystal {
    jni_ref: GlobalRef,
}

impl EndCrystal {
    pub fn new(jni_ref: GlobalRef) -> EndCrystal {
        EndCrystal { jni_ref }
    }

    pub fn from_entity(entity: &crate::mapping::entity::Entity) -> Option<EndCrystal> {
        if entity.instance_of::<EndCrystal>() {
            Some(EndCrystal::new(entity.jni_ref().clone()))
        } else {
            None
        }
    }

    pub fn get_block_pos(&self) -> anyhow::Result<BlockPos> {
        self.in_frame(|| {
            let pos = self.call_method("getBlockPos", &[])?.l()?;
            BlockPos::read(&pos)
        })
    }
}

impl EntityRef for EndCrystal {}
