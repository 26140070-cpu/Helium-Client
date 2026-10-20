use crate::mapping::block::Direction;
use crate::mapping::entity::crystal::EndCrystal;
use crate::mapping::entity::player::Player;
use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::mapping::inventory;
use crate::mapping::math::{BlockPos, Vec3};
use crate::mapping::MappedObject;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
pub struct AutoCrystalModule {
    pub module: ModuleData,
    place_cd: AtomicI32,
    break_cd: AtomicI32,
}

impl AutoCrystalModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::AutoCrystal,
                description: "Places and breaks end crystals".to_string(),
                category: ModuleCategory::Combat,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Slider {
                        name: "PlaceRange".to_string(),
                        value: 4.5,
                        min: 1.0,
                        max: 6.0,
                    },
                    ModuleSetting::Slider {
                        name: "BreakRange".to_string(),
                        value: 4.5,
                        min: 1.0,
                        max: 6.0,
                    },
                    ModuleSetting::Slider {
                        name: "EnemyRange".to_string(),
                        value: 12.0,
                        min: 4.0,
                        max: 24.0,
                    },
                    ModuleSetting::Slider {
                        name: "PlaceDelay".to_string(),
                        value: 0.0,
                        min: 0.0,
                        max: 10.0,
                    },
                    ModuleSetting::Slider {
                        name: "BreakDelay".to_string(),
                        value: 0.0,
                        min: 0.0,
                        max: 10.0,
                    },
                    ModuleSetting::Toggle {
                        name: "Place".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Break".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "PauseOnEat".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Protocol".to_string(),
                        value: false,
                    },
                ],
            },
            place_cd: AtomicI32::new(0),
            break_cd: AtomicI32::new(0),
        }
    }

    fn slider(&self, name: &str, fallback: f32) -> f32 {
        self.module
            .get_setting(name)
            .and_then(|s| s.get_slider_value())
            .unwrap_or(fallback)
    }

    fn toggle(&self, name: &str, fallback: bool) -> bool {
        self.module
            .get_setting(name)
            .and_then(|s| s.get_toggle_value())
            .unwrap_or(fallback)
    }

    fn tick_cd(cd: &AtomicI32) -> bool {
        let v = cd.load(Ordering::SeqCst);
        if v > 0 {
            cd.store(v - 1, Ordering::SeqCst);
            false
        } else {
            true
        }
    }
}

impl Module for AutoCrystalModule {
    fn on_start(&self) -> anyhow::Result<()> {
        self.place_cd.store(0, Ordering::SeqCst);
        self.break_cd.store(0, Ordering::SeqCst);
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
        if self.toggle("PauseOnEat", true) && player.is_using_item()? {
            return Ok(());
        }

        let eye = player.get_eye_position()?;
        let self_id = player.id()?;
        let entities = world.get_entities()?;
        let enemy_range = self.slider("EnemyRange", 12.0) as f64;

        let mut target_pos: Option<Vec3> = None;
        let mut best_enemy_d = enemy_range * enemy_range;
        for e in &entities {
            let Ok(id) = e.id() else { continue };
            if id == self_id {
                continue;
            }
            if !e.instance_of::<Player>() {
                continue;
            }
            if let Some(living) = e.as_living() {
                if !living.is_alive()? {
                    continue;
                }
            }
            let d = e
                .distance_to_sqr(eye.x(), eye.y(), eye.z())
                .unwrap_or(9999.0);
            if d < best_enemy_d {
                best_enemy_d = d;
                target_pos = Some(e.get_position()?);
            }
        }
        let Some(tpos) = target_pos else {
            return Ok(());
        };

        if self.toggle("Break", true) && Self::tick_cd(&self.break_cd) {
            let br = self.slider("BreakRange", 4.5) as f64;
            let br2 = br * br;
            let mut best: Option<crate::mapping::entity::Entity> = None;
            let mut best_d = br2;
            for e in &entities {
                if EndCrystal::from_entity(e).is_none() {
                    continue;
                }
                let d = e
                    .distance_to_sqr(eye.x(), eye.y(), eye.z())
                    .unwrap_or(9999.0);
                if d > br2 {
                    continue;
                }
                let ep = e.get_position()?;
                if (ep.x() - tpos.x()).abs() > 6.0 || (ep.z() - tpos.z()).abs() > 6.0 {
                    continue;
                }
                if d < best_d {
                    best_d = d;
                    best = Some(e.clone());
                }
            }
            if let Some(crystal) = best {
                game_mode.attack(&player, &crystal)?;
                player.swing()?;
                self.break_cd
                    .store(self.slider("BreakDelay", 0.0) as i32, Ordering::SeqCst);
            }
        }

        if self.toggle("Place", true) && Self::tick_cd(&self.place_cd) {
            let crystal_item = inventory::end_crystal()?;
            let inv = player.get_inventory()?;
            let Some(slot) = inv.find_hotbar(&crystal_item)? else {
                return Ok(());
            };
            inv.set_selected(slot)?;

            let pr = self.slider("PlaceRange", 4.5) as f64;
            let protocol = self.toggle("Protocol", false);
            let base = BlockPos::new(
                tpos.x().floor() as i32,
                tpos.y().floor() as i32,
                tpos.z().floor() as i32,
            );
            let mut candidates: Vec<(f64, BlockPos)> = Vec::new();
            for dx in -2..=2 {
                for dz in -2..=2 {
                    for dy in -1..=2 {
                        let pos = base.offset(dx, dy, dz);
                        let cx = pos.x() as f64 + 0.5;
                        let cy = pos.y() as f64 + 1.0;
                        let cz = pos.z() as f64 + 0.5;
                        let dist = (cx - eye.x()).powi(2)
                            + (cy - eye.y()).powi(2)
                            + (cz - eye.z()).powi(2);
                        if dist > pr * pr {
                            continue;
                        }
                        if !world.is_crystal_base(&pos)? {
                            continue;
                        }
                        let above = pos.offset(0, 1, 0);
                        if !world.is_block_air(&above)? {
                            continue;
                        }
                        if protocol {
                            let above2 = pos.offset(0, 2, 0);
                            if !world.is_block_air(&above2)? {
                                continue;
                            }
                        }
                        let score = (cx - tpos.x()).powi(2) + (cz - tpos.z()).powi(2);
                        candidates.push((score, pos));
                    }
                }
            }
            candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            if let Some((_, pos)) = candidates.first() {
                let hit = Vec3::new(
                    pos.x() as f64 + 0.5,
                    pos.y() as f64 + 1.0,
                    pos.z() as f64 + 0.5,
                );
                game_mode.place_block_on(&player, pos, Direction::Up, hit)?;
                player.swing()?;
                self.place_cd
                    .store(self.slider("PlaceDelay", 0.0) as i32, Ordering::SeqCst);
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
