use crate::mapping::component::Component;
use crate::mapping::entity::living::LivingEntity;
use crate::mapping::math::Vec3;
use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JValue};

pub mod crystal;
pub mod living;
pub mod mob;
pub mod player;

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = Entity)]
pub struct Entity {
    jni_ref: GlobalRef,
}

impl Entity {
    pub fn new(jni_ref: GlobalRef) -> Entity {
        Entity { jni_ref }
    }
}

impl EntityRef for Entity {}

pub trait EntityRef: MappedObject + Sized {
    fn as_living(&self) -> Option<LivingEntity> {
        self.instance_of::<LivingEntity>()
            .then(|| LivingEntity::new(self.jni_ref().clone()))
    }

    fn id(&self) -> anyhow::Result<i32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "getId",
                &[],
            )?
            .i()?)
    }

    fn uuid(&self) -> anyhow::Result<String> {
        self.in_frame(|| {
            let uuid = mapping()
                .call_method(
                    MinecraftClassType::Entity,
                    self.jni_ref().as_obj(),
                    "getUUID",
                    &[],
                )?
                .l()?;
            if uuid.is_null() {
                anyhow::bail!("entity UUID is null");
            }
            let mut env = mapping().get_env()?;
            let value = env.call_method(&uuid, "toString", "()Ljava/lang/String;", &[])?;
            mapping().get_string(value.l()?)
        })
    }

    fn get_position(&self) -> anyhow::Result<Vec3> {
        self.in_frame(|| {
            let vec3 = mapping()
                .call_method(
                    MinecraftClassType::Entity,
                    self.jni_ref().as_obj(),
                    "position",
                    &[],
                )?
                .l()?;
            Vec3::read(&vec3)
        })
    }

    fn get_eye_position(&self) -> anyhow::Result<Vec3> {
        self.in_frame(|| {
            let vec3 = mapping()
                .call_method(
                    MinecraftClassType::Entity,
                    self.jni_ref().as_obj(),
                    "getEyePosition",
                    &[],
                )?
                .l()?;
            Vec3::read(&vec3)
        })
    }

    fn get_yaw(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "getYRot",
                &[],
            )?
            .f()?)
    }

    fn get_pitch(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "getXRot",
                &[],
            )?
            .f()?)
    }

    fn distance_to_sqr(&self, x: f64, y: f64, z: f64) -> anyhow::Result<f64> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "distanceToSqr",
                &[JValue::Double(x), JValue::Double(y), JValue::Double(z)],
            )?
            .d()?)
    }

    fn bb_width(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "getBbWidth",
                &[],
            )?
            .f()?)
    }

    fn bb_height(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "getBbHeight",
                &[],
            )?
            .f()?)
    }

    fn is_sprinting(&self) -> anyhow::Result<bool> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "isSprinting",
                &[],
            )?
            .z()?)
    }

    fn set_sprinting(&self, value: bool) -> anyhow::Result<()> {
        mapping().call_method(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "setSprinting",
            &[JValue::from(value)],
        )?;
        Ok(())
    }

    fn on_ground(&self) -> anyhow::Result<bool> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "onGround",
                &[],
            )?
            .z()?)
    }

    fn is_in_water(&self) -> anyhow::Result<bool> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "isInWater",
                &[],
            )?
            .z()?)
    }

    fn is_in_lava(&self) -> anyhow::Result<bool> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "isInLava",
                &[],
            )?
            .z()?)
    }

    fn get_delta_movement(&self) -> anyhow::Result<Vec3> {
        self.in_frame(|| {
            let vec3 = mapping()
                .call_method(
                    MinecraftClassType::Entity,
                    self.jni_ref().as_obj(),
                    "getDeltaMovement",
                    &[],
                )?
                .l()?;
            Vec3::read(&vec3)
        })
    }

    fn set_delta_movement(&self, x: f64, y: f64, z: f64) -> anyhow::Result<()> {
        mapping().call_method(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "setDeltaMovement",
            &[JValue::Double(x), JValue::Double(y), JValue::Double(z)],
        )?;
        Ok(())
    }

    fn set_pos(&self, x: f64, y: f64, z: f64) -> anyhow::Result<()> {
        mapping().call_method(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "setPos",
            &[JValue::Double(x), JValue::Double(y), JValue::Double(z)],
        )?;
        Ok(())
    }

    fn is_shift_key_down(&self) -> anyhow::Result<bool> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "isShiftKeyDown",
                &[],
            )?
            .z()?)
    }

    fn set_no_physics(&self, value: bool) -> anyhow::Result<()> {
        mapping().set_field(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "noPhysics",
            FieldType::Boolean,
            JValue::from(value),
        )?;
        Ok(())
    }

    fn set_invulnerable(&self, value: bool) -> anyhow::Result<()> {
        mapping().call_method(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "setInvulnerable",
            &[JValue::from(value)],
        )?;
        Ok(())
    }

    fn set_rotation(&self, yaw: f32, pitch: f32) -> anyhow::Result<()> {
        mapping().call_method(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "setYRot",
            &[JValue::Float(yaw)],
        )?;
        mapping().call_method(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "setXRot",
            &[JValue::Float(pitch)],
        )?;
        mapping().set_field(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "yRotO",
            FieldType::Float,
            JValue::Float(yaw),
        )?;
        mapping().set_field(
            MinecraftClassType::Entity,
            self.jni_ref().as_obj(),
            "xRotO",
            FieldType::Float,
            JValue::Float(pitch),
        )?;
        Ok(())
    }

    fn get_fall_distance(&self) -> anyhow::Result<f64> {
        Ok(mapping()
            .get_field(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "fallDistance",
                FieldType::Double,
            )?
            .d()?)
    }

    fn reset_fall_distance(&self) -> anyhow::Result<()> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "resetFallDistance",
                &[],
            )?
            .v()?)
    }

    fn get_name(&self) -> anyhow::Result<Component> {
        self.in_frame(|| {
            let component = mapping()
                .call_method(
                    MinecraftClassType::Entity,
                    self.jni_ref().as_obj(),
                    "getName",
                    &[],
                )?
                .l()?;
            Ok(Component::new(mapping().new_global_ref(component)?))
        })
    }

    fn get_tick_count(&self) -> anyhow::Result<i32> {
        Ok(mapping()
            .get_field(
                MinecraftClassType::Entity,
                self.jni_ref().as_obj(),
                "tickCount",
                FieldType::Int,
            )?
            .i()?)
    }
}

pub trait LivingEntityRef: EntityRef {
    fn get_health(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "getHealth",
                &[],
            )?
            .f()?)
    }

    fn get_max_health(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "getMaxHealth",
                &[],
            )?
            .f()?)
    }

    fn is_alive(&self) -> anyhow::Result<bool> {
        Ok(!mapping()
            .call_method(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "isDeadOrDying",
                &[],
            )?
            .z()?)
    }

    fn is_using_item(&self) -> anyhow::Result<bool> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "isUsingItem",
                &[],
            )?
            .z()?)
    }

    fn move_input(&self) -> anyhow::Result<(f32, f32)> {
        let strafe = mapping()
            .get_field(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "xxa",
                FieldType::Float,
            )?
            .f()?;
        let forward = mapping()
            .get_field(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "zza",
                FieldType::Float,
            )?
            .f()?;
        Ok((strafe, forward))
    }

    fn has_move_input(&self) -> anyhow::Result<bool> {
        let (strafe, forward) = self.move_input()?;
        Ok(strafe != 0.0 || forward != 0.0)
    }

    fn get_speed(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "getSpeed",
                &[],
            )?
            .f()?)
    }

    fn is_jumping(&self) -> anyhow::Result<bool> {
        Ok(mapping()
            .get_field(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "jumping",
                FieldType::Boolean,
            )?
            .z()?)
    }

    fn swing(&self) -> anyhow::Result<()> {
        self.in_frame(|| {
            let hand = mapping()
                .get_static_field(
                    MinecraftClassType::InteractionHand,
                    "MAIN_HAND",
                    FieldType::Object(MinecraftClassType::InteractionHand),
                )?
                .l()?;
            mapping().call_method(
                MinecraftClassType::LivingEntity,
                self.jni_ref().as_obj(),
                "swing",
                &[JValue::Object(&hand)],
            )?;
            Ok(())
        })
    }
}

pub trait PlayerRef: LivingEntityRef {
    fn attack_strength_scale(&self) -> anyhow::Result<f32> {
        Ok(mapping()
            .call_method(
                MinecraftClassType::Player,
                self.jni_ref().as_obj(),
                "getAttackStrengthScale",
                &[JValue::Float(0.5)],
            )?
            .f()?)
    }
}
