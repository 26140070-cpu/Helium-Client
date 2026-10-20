use crate::mapping::block::Direction;
use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::mapping::inventory;
use crate::mapping::math::{BlockPos, Vec3};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

#[derive(Debug)]
pub struct SurroundModule {
    pub module: ModuleData,
}

impl SurroundModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Surround,
                description: "Places obsidian around the player's feet".to_string(),
                category: ModuleCategory::World,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Toggle {
                    name: "Double".to_string(),
                    value: false,
                }],
            },
        }
    }

    fn double(&self) -> bool {
        self.module
            .get_setting("Double")
            .and_then(|setting| setting.get_toggle_value())
            .unwrap_or(false)
    }
}

impl Module for SurroundModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let mc = minecraft();
        if mc.current_screen().is_open() {
            return Ok(());
        }
        let (Some(player), Some(world), Some(game_mode)) =
            (mc.player()?, mc.world()?, mc.game_mode()?)
        else {
            return Ok(());
        };

        let position = player.get_position()?;
        let feet = BlockPos::new(
            position.x().floor() as i32,
            position.y().floor() as i32 - 1,
            position.z().floor() as i32,
        );
        let inventory = player.get_inventory()?;
        let obsidian = inventory::obsidian()?;
        let Some(slot) = inventory.find_hotbar(&obsidian)? else {
            return Ok(());
        };

        if inventory.selected()? != slot {
            inventory.set_selected(slot)?;
        }

        let directions = [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];
        for direction in directions {
            let (dx, _, dz) = direction.offset();
            let target = feet.offset(dx, 0, dz);
            if !world.is_block_air(&target)? {
                continue;
            }
            place_at(&world, &game_mode, &player, target)?;
            if self.double() {
                let upper = target.offset(0, 1, 0);
                if world.is_block_air(&upper)? {
                    place_at(&world, &game_mode, &player, upper)?;
                }
            }
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

fn place_at(
    world: &crate::mapping::client::world::World,
    game_mode: &crate::mapping::client::gamemode::MultiPlayerGameMode,
    player: &crate::mapping::entity::player::LocalPlayer,
    target: BlockPos,
) -> anyhow::Result<()> {
    if let Some((anchor, face)) = find_anchor(world, target)? {
        game_mode.place_block_on(player, &anchor, face, face_center(anchor, face))?;
        player.swing()?;
    }
    Ok(())
}

fn find_anchor(
    world: &crate::mapping::client::world::World,
    target: BlockPos,
) -> anyhow::Result<Option<(BlockPos, Direction)>> {
    for direction in Direction::ALL {
        let (dx, dy, dz) = direction.offset();
        let anchor = target.offset(dx, dy, dz);
        if !world.is_block_air(&anchor)? {
            return Ok(Some((anchor, direction.opposite())));
        }
    }
    Ok(None)
}

fn face_center(anchor: BlockPos, face: Direction) -> Vec3 {
    let (fx, fy, fz) = face.offset();
    Vec3::new(
        anchor.x() as f64 + 0.5 + fx as f64 * 0.5,
        anchor.y() as f64 + 0.5 + fy as f64 * 0.5,
        anchor.z() as f64 + 0.5 + fz as f64 * 0.5,
    )
}
