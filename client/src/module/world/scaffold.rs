use crate::mapping::block::Direction;
use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::mapping::math::{BlockPos, Vec3};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

const MOVING_THRESHOLD: f64 = 0.01;

#[derive(Debug)]
pub struct ScaffoldModule {
    pub module: ModuleData,
}

impl ScaffoldModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Scaffold,
                description: "Bridges blocks ahead of the player".to_string(),
                category: ModuleCategory::World,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Toggle {
                    name: "Tower".to_string(),
                    value: true,
                }],
            },
        }
    }
}

impl Module for ScaffoldModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let minecraft = minecraft();
        if minecraft.current_screen().is_open() {
            return Ok(());
        }
        let (Some(player), Some(world), Some(game_mode)) = (
            minecraft.player()?,
            minecraft.world()?,
            minecraft.game_mode()?,
        ) else {
            return Ok(());
        };

        let feet = player.get_position()?;
        let support = BlockPos::new(
            feet.x().floor() as i32,
            feet.y().round() as i32 - 1,
            feet.z().floor() as i32,
        );

        let tower = self
            .module
            .get_setting("Tower")
            .and_then(|s| s.get_toggle_value())
            .unwrap_or(true);

        let target = if world.is_block_air(&support)? {
            support
        } else if tower && player.is_jumping()? {
            support.offset(0, 1, 0)
        } else {
            let motion = player.get_delta_movement()?;
            let Some((dx, dz)) = dominant_step(motion.x(), motion.z()) else {
                return Ok(());
            };
            support.offset(dx, 0, dz)
        };

        if !world.is_block_air(&target)? {
            return Ok(());
        }

        for direction in Direction::ALL {
            let (dx, dy, dz) = direction.offset();
            let anchor = target.offset(dx, dy, dz);
            if world.is_block_air(&anchor)? {
                continue;
            }
            let face = direction.opposite();
            return game_mode.place_block_on(&player, &anchor, face, face_center(anchor, face));
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

fn dominant_step(vx: f64, vz: f64) -> Option<(i32, i32)> {
    if vx.abs() < MOVING_THRESHOLD && vz.abs() < MOVING_THRESHOLD {
        return None;
    }
    if vx.abs() >= vz.abs() {
        Some((vx.signum() as i32, 0))
    } else {
        Some((0, vz.signum() as i32))
    }
}

fn face_center(anchor: BlockPos, face: Direction) -> Vec3 {
    let (fx, fy, fz) = face.offset();
    Vec3::new(
        anchor.x() as f64 + 0.5 + fx as f64 * 0.5,
        anchor.y() as f64 + 0.5 + fy as f64 * 0.5,
        anchor.z() as f64 + 0.5 + fz as f64 * 0.5,
    )
}
