use crate::mapping::entity::{EntityRef, LivingEntityRef, PlayerRef};
use crate::mapping::math::Vec3;
use crate::mapping::MappedObject;
use crate::module::combat::rotation::Rotation;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
pub struct TriggerBotModule {
    pub module: ModuleData,
    cooldown: AtomicI32,
}

impl TriggerBotModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::TriggerBot,
                description: "Attacks the entity under your crosshair".to_string(),
                category: ModuleCategory::Combat,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Slider {
                        name: "Range".to_string(),
                        value: 3.2,
                        min: 2.0,
                        max: 6.0,
                    },
                    ModuleSetting::Slider {
                        name: "FOV".to_string(),
                        value: 8.0,
                        min: 1.0,
                        max: 30.0,
                    },
                    ModuleSetting::Toggle {
                        name: "Players".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Mobs".to_string(),
                        value: true,
                    },
                ],
            },
            cooldown: AtomicI32::new(0),
        }
    }
}

impl Module for TriggerBotModule {
    fn on_start(&self) -> anyhow::Result<()> {
        self.cooldown.store(0, Ordering::SeqCst);
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let cd = self.cooldown.load(Ordering::SeqCst);
        if cd > 0 {
            self.cooldown.store(cd - 1, Ordering::SeqCst);
            return Ok(());
        }
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
        if player.attack_strength_scale()? < 0.95 {
            return Ok(());
        }
        let range = self
            .module
            .get_setting("Range")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(3.2) as f64;
        let fov = self
            .module
            .get_setting("FOV")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(8.0);
        let hit_players = self
            .module
            .get_setting("Players")
            .and_then(|s| s.get_toggle_value())
            .unwrap_or(true);
        let hit_mobs = self
            .module
            .get_setting("Mobs")
            .and_then(|s| s.get_toggle_value())
            .unwrap_or(true);
        let self_id = player.id()?;
        let entities = world.get_entities()?;
        let mut best = None;
        let mut best_angle = fov;
        for entity in &entities {
            let Ok(id) = entity.id() else { continue };
            if id == self_id {
                continue;
            }
            let is_player = entity.instance_of::<crate::mapping::entity::player::Player>();
            let is_mob = entity.as_living().is_some() && !is_player;
            if is_player && !hit_players {
                continue;
            }
            if is_mob && !hit_mobs {
                continue;
            }
            if !is_player && !is_mob {
                continue;
            }
            if let Some(living) = entity.as_living() {
                if !living.is_alive()? {
                    continue;
                }
            }
            let eye = player.get_eye_position()?;
            let dist = entity
                .distance_to_sqr(eye.x(), eye.y(), eye.z())
                .unwrap_or(999.0);
            if dist > range * range {
                continue;
            }
            let feet = entity.get_position()?;
            let height = entity.bb_height().unwrap_or(1.8) as f64;
            let aim = Vec3::new(feet.x(), feet.y() + height * 0.7, feet.z());
            let target_rot = Rotation::towards(eye, aim);
            let current = Rotation::new(player.get_yaw()?, player.get_pitch()?);
            let angle = current.angle_to(target_rot);
            if angle <= best_angle {
                best_angle = angle;
                best = Some(entity.clone());
            }
        }
        if let Some(target) = best {
            if best_angle <= fov {
                game_mode.attack(&player, &target)?;
                player.swing()?;
                self.cooldown.store(2, Ordering::SeqCst);
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
