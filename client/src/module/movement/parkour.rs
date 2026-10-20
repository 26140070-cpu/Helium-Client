use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::mapping::math::BlockPos;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::state::minecraft;

#[derive(Debug)]
pub struct ParkourModule {
    pub module: ModuleData,
}

impl ParkourModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Parkour,
                description: "Auto-jumps at block edges".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
        }
    }
}

impl Module for ParkourModule {
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
        if !player.on_ground()? || !player.has_move_input()? {
            return Ok(());
        }
        let feet = player.get_position()?;
        let motion = player.get_delta_movement()?;
        let ahead = BlockPos::new(
            (feet.x()
                + motion.x().signum().max(0.0) * 0.4
                + if motion.x().abs() < 0.01 {
                    0.0
                } else {
                    motion.x().signum() * 0.2
                })
            .floor() as i32,
            feet.y().round() as i32 - 1,
            (feet.z()
                + motion.z().signum().max(0.0) * 0.4
                + if motion.z().abs() < 0.01 {
                    0.0
                } else {
                    motion.z().signum() * 0.2
                })
            .floor() as i32,
        );
        let under = BlockPos::new(
            feet.x().floor() as i32,
            feet.y().round() as i32 - 1,
            feet.z().floor() as i32,
        );
        if !world.is_block_air(&under)? && world.is_block_air(&ahead)? {
            player.set_delta_movement(motion.x(), 0.42, motion.z())?;
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
