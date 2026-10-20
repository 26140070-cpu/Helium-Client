pub mod aimbot;
pub mod aura;
pub mod autocrystal;
pub mod criticals;
pub mod killaura;
pub mod mobaura;
pub mod reach;
pub mod rotation;
pub mod triggerbot;
pub mod velocity;

pub use autocrystal::AutoCrystalModule;
pub use reach::ReachModule;
pub use triggerbot::TriggerBotModule;

use crate::mapping::entity::player::LocalPlayer;
use crate::mapping::entity::{Entity, EntityRef};
use crate::mapping::math::Vec3;
use rotation::Rotation;

pub fn look_at(
    player: &LocalPlayer,
    target: &Entity,
    speed: f32,
    max_fov: f32,
) -> anyhow::Result<f32> {
    let eye = player.get_eye_position()?;
    let feet = target.get_position()?;
    let height = target.bb_height()? as f64;

    let aim = Vec3::new(feet.x(), feet.y() + height * 0.7, feet.z());
    let target_rotation = Rotation::towards(eye, aim);

    let current = Rotation::new(player.get_yaw()?, player.get_pitch()?);
    let angle = current.angle_to(target_rotation);
    if angle <= max_fov {
        rotation::aim(target_rotation, speed);
    }
    Ok(angle)
}

pub fn pick_target(
    entities: &[Entity],
    eye: Vec3,
    range_sq: f64,
    exclude_id: i32,
    locked: Option<i32>,
    accept: impl Fn(&Entity) -> bool,
) -> Option<(i32, Entity)> {
    let mut candidates: Vec<(i32, Entity, f64)> = Vec::new();
    for entity in entities {
        let distance = match entity.distance_to_sqr(eye.x(), eye.y(), eye.z()) {
            Ok(distance) if distance <= range_sq => distance,
            _ => continue,
        };
        if !accept(entity) {
            continue;
        }
        let Ok(id) = entity.id() else {
            continue;
        };
        if id == exclude_id {
            continue;
        }
        candidates.push((id, entity.clone(), distance));
    }

    if let Some(locked) = locked {
        if let Some((id, entity, _)) = candidates.iter().find(|(id, _, _)| *id == locked) {
            return Some((*id, entity.clone()));
        }
    }
    candidates
        .into_iter()
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(id, entity, _)| (id, entity))
}
