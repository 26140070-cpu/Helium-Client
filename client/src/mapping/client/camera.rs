use crate::mapping::math::Vec3;
use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use jni::objects::GlobalRef;

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = Camera)]
pub struct Camera {
    jni_ref: GlobalRef,
}

impl Camera {
    pub fn new(jni_ref: GlobalRef) -> Camera {
        Camera { jni_ref }
    }

    pub fn position(&self) -> anyhow::Result<Vec3> {
        self.in_frame(|| {
            let vec3 = self
                .get_field("position", FieldType::Object(MinecraftClassType::Vec3))?
                .l()?;
            Vec3::read(&vec3)
        })
    }

    pub fn yaw(&self) -> anyhow::Result<f32> {
        Ok(self.get_field("yRot", FieldType::Float)?.f()?)
    }

    pub fn pitch(&self) -> anyhow::Result<f32> {
        Ok(self.get_field("xRot", FieldType::Float)?.f()?)
    }

    pub fn fov(&self) -> anyhow::Result<f32> {
        Ok(self.get_field("fov", FieldType::Float)?.f()?)
    }
}
