use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JValue};

#[derive(Debug, MappedObject)]
#[mapped(class = Options)]
pub struct Options {
    jni_ref: GlobalRef,
}

impl Options {
    pub fn new(jni_ref: GlobalRef) -> Options {
        Options { jni_ref }
    }

    pub fn fov(&self) -> anyhow::Result<OptionInstance> {
        self.in_frame(|| {
            let option = self
                .get_field("fov", FieldType::Object(MinecraftClassType::OptionInstance))?
                .l()?;
            Ok(OptionInstance::new(mapping().new_global_ref(option)?))
        })
    }

    pub fn fov_effect_scale(&self) -> anyhow::Result<OptionInstance> {
        self.in_frame(|| {
            let option = self
                .get_field(
                    "fovEffectScale",
                    FieldType::Object(MinecraftClassType::OptionInstance),
                )?
                .l()?;
            Ok(OptionInstance::new(mapping().new_global_ref(option)?))
        })
    }

    pub fn gamma(&self) -> anyhow::Result<OptionInstance> {
        self.in_frame(|| {
            let option = self
                .get_field(
                    "gamma",
                    FieldType::Object(MinecraftClassType::OptionInstance),
                )?
                .l()?;
            Ok(OptionInstance::new(mapping().new_global_ref(option)?))
        })
    }

    pub fn sensitivity(&self) -> anyhow::Result<OptionInstance> {
        self.in_frame(|| {
            let option = self
                .get_field(
                    "sensitivity",
                    FieldType::Object(MinecraftClassType::OptionInstance),
                )?
                .l()?;
            Ok(OptionInstance::new(mapping().new_global_ref(option)?))
        })
    }

    pub fn screen_effect_scale(&self) -> anyhow::Result<OptionInstance> {
        self.in_frame(|| {
            let option = self
                .get_field(
                    "screenEffectScale",
                    FieldType::Object(MinecraftClassType::OptionInstance),
                )?
                .l()?;
            Ok(OptionInstance::new(mapping().new_global_ref(option)?))
        })
    }

    pub fn damage_tilt_strength(&self) -> anyhow::Result<OptionInstance> {
        self.in_frame(|| {
            let option = self
                .get_field(
                    "damageTiltStrength",
                    FieldType::Object(MinecraftClassType::OptionInstance),
                )?
                .l()?;
            Ok(OptionInstance::new(mapping().new_global_ref(option)?))
        })
    }
}

#[derive(Debug, MappedObject)]
#[mapped(class = OptionInstance)]
pub struct OptionInstance {
    jni_ref: GlobalRef,
}

impl OptionInstance {
    pub fn new(jni_ref: GlobalRef) -> OptionInstance {
        OptionInstance { jni_ref }
    }

    pub fn get_int(&self) -> anyhow::Result<i32> {
        self.in_frame(|| {
            let value = self.call_method("get", &[])?.l()?;
            Ok(mapping()
                .call_method(MinecraftClassType::Integer, &value, "intValue", &[])?
                .i()?)
        })
    }

    pub fn get_double(&self) -> anyhow::Result<f64> {
        self.in_frame(|| {
            let value = self.call_method("get", &[])?.l()?;
            Ok(mapping()
                .call_method(MinecraftClassType::Double, &value, "doubleValue", &[])?
                .d()?)
        })
    }

    pub fn force_double(&self, value: f64) -> anyhow::Result<()> {
        self.in_frame(|| {
            let mut env = mapping().get_env()?;
            let double_class = env.find_class("java/lang/Double")?;
            let boxed = env.new_object(&double_class, "(D)V", &[JValue::Double(value)])?;
            self.set_field(
                "value",
                FieldType::Object(MinecraftClassType::Object),
                JValue::Object(&boxed),
            )
        })
    }
}
