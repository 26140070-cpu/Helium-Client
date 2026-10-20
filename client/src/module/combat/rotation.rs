use crate::mapping::entity::EntityRef;
use crate::mapping::math::Vec3;
use crate::state::minecraft;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TARGET_TIMEOUT: Duration = Duration::from_millis(150);

const MAX_FRAME_DT: f32 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl Rotation {
    pub const fn new(yaw: f32, pitch: f32) -> Rotation {
        Rotation { yaw, pitch }
    }

    pub fn towards(from: Vec3, to: Vec3) -> Rotation {
        let dx = to.x() - from.x();
        let dy = to.y() - from.y();
        let dz = to.z() - from.z();
        let ground = (dx * dx + dz * dz).sqrt();
        Rotation {
            yaw: wrap_degrees((dz.atan2(dx).to_degrees() - 90.0) as f32),
            pitch: (-dy.atan2(ground).to_degrees() as f32).clamp(-90.0, 90.0),
        }
    }

    pub fn angle_to(self, other: Rotation) -> f32 {
        let dyaw = wrap_degrees(other.yaw - self.yaw);
        let dpitch = other.pitch - self.pitch;
        (dyaw * dyaw + dpitch * dpitch).sqrt()
    }

    fn lerp_towards(self, target: Rotation, alpha: f32) -> Rotation {
        let dyaw = wrap_degrees(target.yaw - self.yaw);
        let dpitch = target.pitch - self.pitch;
        Rotation {
            yaw: self.yaw + dyaw * alpha,
            pitch: (self.pitch + dpitch * alpha).clamp(-90.0, 90.0),
        }
    }
}

pub fn wrap_degrees(mut angle: f32) -> f32 {
    angle %= 360.0;
    if angle >= 180.0 {
        angle -= 360.0;
    } else if angle < -180.0 {
        angle += 360.0;
    }
    angle
}

#[derive(Debug, Clone, Copy)]
struct Target {
    rotation: Rotation,

    speed: f32,

    refreshed: Instant,
}

struct RotationController {
    target: Option<Target>,
    last_frame: Option<Instant>,
}

impl RotationController {
    const fn new() -> RotationController {
        RotationController {
            target: None,
            last_frame: None,
        }
    }
}

static CONTROLLER: Mutex<RotationController> = Mutex::new(RotationController::new());

pub fn aim(rotation: Rotation, speed: f32) {
    if let Ok(mut controller) = CONTROLLER.lock() {
        controller.target = Some(Target {
            rotation,
            speed,
            refreshed: Instant::now(),
        });
    }
}

pub fn update() {
    let step = {
        let Ok(mut controller) = CONTROLLER.lock() else {
            return;
        };
        let now = Instant::now();
        let dt = controller
            .last_frame
            .map_or(0.0, |last| (now - last).as_secs_f32())
            .min(MAX_FRAME_DT);
        controller.last_frame = Some(now);

        match controller.target {
            Some(target) if target.refreshed.elapsed() <= TARGET_TIMEOUT => Some((target, dt)),
            _ => {
                controller.target = None;
                None
            }
        }
    };

    let Some((target, dt)) = step else {
        return;
    };
    if dt <= 0.0 {
        return;
    }

    let _ = ease_camera(target, dt);
}

fn ease_camera(target: Target, dt: f32) -> anyhow::Result<()> {
    let Some(player) = minecraft().player()? else {
        return Ok(());
    };
    let current = Rotation::new(player.get_yaw()?, player.get_pitch()?);

    let alpha = 1.0 - (-target.speed * dt).exp();
    let next = current.lerp_towards(target.rotation, alpha);

    player.set_rotation(next.yaw, next.pitch)
}
