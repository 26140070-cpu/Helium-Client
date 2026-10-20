pub mod clientbound_player_info_update;
pub mod clientbound_set_entity_motion;
pub mod serverbound_move_player;

use crate::mapping::MinecraftClassType;
use crate::state::mapping;
use clientbound_player_info_update::ClientboundPlayerInfoUpdatePacket;
use clientbound_set_entity_motion::ClientboundSetEntityMotionPacket;
use jni::objects::JObject;
use jni::sys::jobject;
use jni::JNIEnv;
use serverbound_move_player::ServerboundMovePlayerPacket;

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    ServerboundMovePlayer(ServerboundMovePlayerPacket),

    ClientboundSetEntityMotion(ClientboundSetEntityMotionPacket),

    ClientboundPlayerInfoUpdate(ClientboundPlayerInfoUpdatePacket),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PacketAction {
    #[default]
    Forward,

    Cancel,
}

impl Packet {
    pub fn from_outbound(env: &mut JNIEnv, packet: &JObject) -> anyhow::Result<Option<Packet>> {
        if is_instance(env, packet, serverbound_move_player::CLASS_TYPE)? {
            return Ok(Some(Packet::ServerboundMovePlayer(
                ServerboundMovePlayerPacket::read(env, packet)?,
            )));
        }
        Ok(None)
    }

    pub fn from_inbound(env: &mut JNIEnv, packet: &JObject) -> anyhow::Result<Option<Packet>> {
        if is_instance(env, packet, clientbound_set_entity_motion::CLASS_TYPE)? {
            return Ok(Some(Packet::ClientboundSetEntityMotion(
                ClientboundSetEntityMotionPacket::read(env, packet)?,
            )));
        }
        if is_instance(env, packet, clientbound_player_info_update::CLASS_TYPE)? {
            return Ok(Some(Packet::ClientboundPlayerInfoUpdate(
                ClientboundPlayerInfoUpdatePacket {
                    jni_ref: env.new_global_ref(packet)?,
                },
            )));
        }
        Ok(None)
    }

    pub fn to_java(self, env: &mut JNIEnv) -> anyhow::Result<jobject> {
        match self {
            Packet::ServerboundMovePlayer(packet) => packet.to_java(env),
            Packet::ClientboundSetEntityMotion(packet) => packet.to_java(env),

            Packet::ClientboundPlayerInfoUpdate(packet) => {
                Ok(env.new_local_ref(packet.jni_ref.as_obj())?.into_raw())
            }
        }
    }
}

fn is_instance(
    env: &mut JNIEnv,
    object: &JObject,
    class: MinecraftClassType,
) -> anyhow::Result<bool> {
    let jclass = mapping().resolve_class(env, class.get_name())?;
    Ok(env.is_instance_of(object, &jclass)?)
}
