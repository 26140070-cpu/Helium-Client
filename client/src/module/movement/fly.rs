use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;

const VANILLA_FLY_SPEED: f32 = 0.05;

#[derive(Debug)]
pub struct FlyModule {
    pub module: ModuleData,
}

impl FlyModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Fly,
                description: "Enables flight".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyF,
                enabled: false,
                settings: vec![
                    ModuleSetting::Slider {
                        name: "Speed".to_string(),
                        value: 1.0,
                        min: 0.1,
                        max: 5.0,
                    },
                    ModuleSetting::Slider {
                        name: "Vertical".to_string(),
                        value: 0.4,
                        min: 0.1,
                        max: 2.0,
                    },
                ],
            },
        }
    }

    fn fly_speed(&self) -> f32 {
        let multiplier = self
            .module
            .get_setting("Speed")
            .and_then(|setting| setting.get_slider_value())
            .unwrap_or(1.0);
        VANILLA_FLY_SPEED * multiplier
    }

    fn vertical(&self) -> f64 {
        self.module
            .get_setting("Vertical")
            .and_then(|s| s.get_slider_value())
            .unwrap_or(0.4) as f64
    }
}

impl Module for FlyModule {
    fn on_start(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            player.abilities.fly(true)?;
            player.abilities.set_flying_speed(self.fly_speed())?;
        }
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            player.abilities.fly(false)?;
            player.abilities.set_flying_speed(VANILLA_FLY_SPEED)?;
        }
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        if let Some(player) = minecraft().player()? {
            player.abilities.fly(true)?;
            player.abilities.set_flying_speed(self.fly_speed())?;
            let motion = player.get_delta_movement()?;
            let v = self.vertical();
            if player.is_jumping()? {
                player.set_delta_movement(motion.x(), v, motion.z())?;
            } else if player.is_shift_key_down()? {
                player.set_delta_movement(motion.x(), -v, motion.z())?;
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
