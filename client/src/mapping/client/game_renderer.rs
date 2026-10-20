use crate::mapping::client::camera::Camera;
use crate::mapping::MappedObject;
use crate::state::mapping;
use jni::objects::GlobalRef;

#[derive(Debug, MappedObject)]
#[mapped(class = GameRenderer)]
pub struct GameRenderer {
    jni_ref: GlobalRef,
}

impl GameRenderer {
    pub fn new(jni_ref: GlobalRef) -> GameRenderer {
        GameRenderer { jni_ref }
    }

    pub fn get_main_camera(&self) -> anyhow::Result<Camera> {
        self.in_frame(|| {
            let camera = self.call_method("getMainCamera", &[])?.l()?;
            Ok(Camera::new(mapping().new_global_ref(camera)?))
        })
    }
}
