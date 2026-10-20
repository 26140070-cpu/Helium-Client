use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

#[derive(Debug)]
pub struct StrafeModule {
    pub module: ModuleData,
}

impl StrafeModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Strafe,
                description: "Air strafe control".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![ModuleSetting::Slider {
                    name: "Speed".to_string(),
                    value: 0.28,
                    min: 0.1,
                    max: 0.6,
                }],
            },
        }
    }
}

impl Module for StrafeModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let Some(player) = minecraft().player()? else {
            return Ok(());
        };
        if player.on_ground()? || !player.has_move_input()? {
            return Ok(());
        }
        let speed = self
            .module
            .get_setting("Speed")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(0.28) as f64;
        let yaw = player.get_yaw()?.to_radians() as f64;
        let (strafe, forward) = player.move_input()?;
        let mut mx = 0.0_f64;
        let mut mz = 0.0_f64;
        if forward != 0.0 {
            mx -= yaw.sin() * forward as f64;
            mz += yaw.cos() * forward as f64;
        }
        if strafe != 0.0 {
            mx -= (yaw + std::f64::consts::FRAC_PI_2).sin() * strafe as f64;
            mz += (yaw + std::f64::consts::FRAC_PI_2).cos() * strafe as f64;
        }
        let len = (mx * mx + mz * mz).sqrt();
        if len > 0.0 {
            mx = mx / len * speed;
            mz = mz / len * speed;
            let motion = player.get_delta_movement()?;
            player.set_delta_movement(mx, motion.y(), mz)?;
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
