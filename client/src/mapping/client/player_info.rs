use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JValue};

#[derive(Debug, MappedObject)]
#[mapped(class = PlayerInfo)]
pub struct PlayerInfo {
    jni_ref: GlobalRef,
}

impl PlayerInfo {
    pub fn new(jni_ref: GlobalRef) -> PlayerInfo {
        PlayerInfo { jni_ref }
    }

    pub fn get_game_mode(&self) -> anyhow::Result<GlobalRef> {
        self.in_frame(|| {
            let mode = self
                .get_field("gameMode", FieldType::Object(MinecraftClassType::GameType))?
                .l()?;
            mapping().new_global_ref(mode)
        })
    }

    pub fn set_game_mode(&self, mode: &GlobalRef) -> anyhow::Result<()> {
        self.set_field(
            "gameMode",
            FieldType::Object(MinecraftClassType::GameType),
            JValue::Object(mode.as_obj()),
        )?;
        Ok(())
    }
}

pub fn game_type_spectator() -> anyhow::Result<GlobalRef> {
    mapping().in_frame(|| {
        let obj = mapping()
            .get_static_field(
                MinecraftClassType::GameType,
                "SPECTATOR",
                FieldType::Object(MinecraftClassType::GameType),
            )?
            .l()?;
        mapping().new_global_ref(obj)
    })
}
