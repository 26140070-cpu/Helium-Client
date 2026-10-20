use crate::mapping::block::Direction;
use crate::mapping::entity::EntityRef;
use crate::mapping::math::BlockPos;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::state::minecraft;

#[derive(Debug)]
pub struct SafeWalkModule {
    pub module: ModuleData,
}

impl SafeWalkModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::SafeWalk,
                description: "Prevents walking off edges".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
        }
    }
}

impl Module for SafeWalkModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let minecraft = minecraft();
        let (Some(player), Some(world)) = (minecraft.player()?, minecraft.world()?) else {
            return Ok(());
        };
        if !player.on_ground()? {
            return Ok(());
        }
        let feet = player.get_position()?;
        let support = BlockPos::new(
            feet.x().floor() as i32,
            feet.y().round() as i32 - 1,
            feet.z().floor() as i32,
        );
        let motion = player.get_delta_movement()?;
        if motion.x().abs() < 0.001 && motion.z().abs() < 0.001 {
            return Ok(());
        }
        let next = BlockPos::new(
            (feet.x() + motion.x().signum() * 0.3).floor() as i32,
            support.y(),
            (feet.z() + motion.z().signum() * 0.3).floor() as i32,
        );
        if world.is_block_air(&next)? {
            player.set_delta_movement(0.0, motion.y(), 0.0)?;
            let _ = Direction::ALL;
        }
        Ok(())
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
