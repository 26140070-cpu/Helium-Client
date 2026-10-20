use crate::mapping::block::Direction;
use crate::mapping::entity::player::LocalPlayer;
use crate::mapping::entity::Entity;
use crate::mapping::math::{BlockPos, Vec3};
use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JValue};

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = MultiPlayerGameMode)]
pub struct MultiPlayerGameMode {
    jni_ref: GlobalRef,
}

impl MultiPlayerGameMode {
    pub fn new(jni_ref: GlobalRef) -> Self {
        Self { jni_ref }
    }

    pub fn attack(&self, player: &LocalPlayer, target: &Entity) -> anyhow::Result<()> {
        self.call_method(
            "attack",
            &[
                JValue::Object(player.jni_ref().as_obj()),
                JValue::Object(target.jni_ref().as_obj()),
            ],
        )?;
        Ok(())
    }

    pub fn use_main_hand(&self, player: &LocalPlayer) -> anyhow::Result<()> {
        self.in_frame(|| {
            let hand = mapping()
                .get_static_field(
                    MinecraftClassType::InteractionHand,
                    "MAIN_HAND",
                    FieldType::Object(MinecraftClassType::InteractionHand),
                )?
                .l()?;
            self.call_method(
                "useItem",
                &[
                    JValue::Object(player.jni_ref().as_obj()),
                    JValue::Object(&hand),
                ],
            )?;
            Ok(())
        })
    }

    fn container_click(
        &self,
        container_id: i32,
        slot: i32,
        button: i32,
        action: &str,
        player: &LocalPlayer,
    ) -> anyhow::Result<()> {
        self.in_frame(|| {
            let input = mapping()
                .get_static_field(
                    MinecraftClassType::ContainerInput,
                    action,
                    FieldType::Object(MinecraftClassType::ContainerInput),
                )?
                .l()?;
            self.call_method(
                "handleContainerInput",
                &[
                    JValue::Int(container_id),
                    JValue::Int(slot),
                    JValue::Int(button),
                    JValue::Object(&input),
                    JValue::Object(player.jni_ref().as_obj()),
                ],
            )?;
            Ok(())
        })
    }

    pub fn container_swap(
        &self,
        container_id: i32,
        slot: i32,
        hotbar_button: i32,
        player: &LocalPlayer,
    ) -> anyhow::Result<()> {
        self.container_click(container_id, slot, hotbar_button, "SWAP", player)
    }

    pub fn container_quick_move(
        &self,
        container_id: i32,
        slot: i32,
        player: &LocalPlayer,
    ) -> anyhow::Result<()> {
        self.container_click(container_id, slot, 0, "QUICK_MOVE", player)
    }

    fn destroy_block(&self, method: &str, pos: &BlockPos, face: Direction) -> anyhow::Result<()> {
        self.in_frame(|| {
            let mut env = mapping().get_env()?;
            let pos_obj = pos.to_java(&mut env)?;
            let face_obj = face.to_java()?;
            self.call_method(
                method,
                &[JValue::Object(&pos_obj), JValue::Object(&face_obj)],
            )?;
            Ok(())
        })
    }

    pub fn start_destroy_block(&self, pos: &BlockPos, face: Direction) -> anyhow::Result<()> {
        self.destroy_block("startDestroyBlock", pos, face)
    }

    pub fn continue_destroy_block(&self, pos: &BlockPos, face: Direction) -> anyhow::Result<()> {
        self.destroy_block("continueDestroyBlock", pos, face)
    }

    pub fn stop_destroy_block(&self) -> anyhow::Result<()> {
        self.call_method("stopDestroyBlock", &[])?;
        Ok(())
    }

    pub fn place_block_on(
        &self,
        player: &LocalPlayer,
        anchor: &BlockPos,
        face: Direction,
        hit: Vec3,
    ) -> anyhow::Result<()> {
        self.in_frame(|| {
            let mut env = mapping().get_env()?;
            let hit_obj = hit.to_java(&mut env)?;
            let anchor_obj = anchor.to_java(&mut env)?;
            let face_obj = face.to_java()?;
            let block_hit_class =
                mapping().resolve_class(&mut env, MinecraftClassType::BlockHitResult.get_name())?;
            let block_hit = env.new_object(
                &block_hit_class,
                "(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/Direction;\
                 Lnet/minecraft/core/BlockPos;Z)V",
                &[
                    JValue::Object(&hit_obj),
                    JValue::Object(&face_obj),
                    JValue::Object(&anchor_obj),
                    JValue::Bool(0),
                ],
            )?;
            let hand = mapping()
                .get_static_field(
                    MinecraftClassType::InteractionHand,
                    "MAIN_HAND",
                    FieldType::Object(MinecraftClassType::InteractionHand),
                )?
                .l()?;
            self.call_method(
                "useItemOn",
                &[
                    JValue::Object(player.jni_ref().as_obj()),
                    JValue::Object(&hand),
                    JValue::Object(&block_hit),
                ],
            )?;
            Ok(())
        })
    }
}
