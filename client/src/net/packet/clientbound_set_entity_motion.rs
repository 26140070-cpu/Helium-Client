use crate::mapping::MinecraftClassType;
use crate::state::mapping;
use jni::objects::{JObject, JValue};
use jni::sys::jobject;
use jni::JNIEnv;

pub const CLASS_TYPE: MinecraftClassType = MinecraftClassType::ClientboundSetEntityMotionPacket;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetEntityMotionPacket {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ClientboundSetEntityMotionPacket {
    pub fn read(
        env: &mut JNIEnv,
        packet: &JObject,
    ) -> anyhow::Result<ClientboundSetEntityMotionPacket> {
        let entity_id = env.call_method(packet, "id", "()I", &[])?.i()?;
        let movement = env
            .call_method(packet, "movement", "()Lnet/minecraft/world/phys/Vec3;", &[])?
            .l()?;
        Ok(ClientboundSetEntityMotionPacket {
            entity_id,
            x: env.get_field(&movement, "x", "D")?.d()?,
            y: env.get_field(&movement, "y", "D")?.d()?,
            z: env.get_field(&movement, "z", "D")?.d()?,
        })
    }

    pub fn to_java(self, env: &mut JNIEnv) -> anyhow::Result<jobject> {
        let vec3_class = mapping().resolve_class(env, MinecraftClassType::Vec3.get_name())?;
        let movement = env.new_object(
            &vec3_class,
            "(DDD)V",
            &[
                JValue::Double(self.x),
                JValue::Double(self.y),
                JValue::Double(self.z),
            ],
        )?;

        let packet_class = mapping().resolve_class(env, CLASS_TYPE.get_name())?;
        let packet = env.new_object(
            &packet_class,
            "(ILnet/minecraft/world/phys/Vec3;)V",
            &[JValue::Int(self.entity_id), JValue::Object(&movement)],
        )?;
        Ok(packet.into_raw())
    }
}
