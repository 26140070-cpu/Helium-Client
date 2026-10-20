use crate::mapping::entity::LivingEntityRef;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::minecraft;
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
pub struct AutoEatModule {
    pub module: ModuleData,
    previous_slot: AtomicI32,
    eating_slot: AtomicI32,
}

impl AutoEatModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::AutoEat,
                description: "Automatically eats food when health or hunger is low".to_string(),
                category: ModuleCategory::Player,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Slider {
                        name: "Health Threshold".to_string(),
                        value: 10.0,
                        min: 1.0,
                        max: 20.0,
                    },
                    ModuleSetting::Slider {
                        name: "Hunger Threshold".to_string(),
                        value: 16.0,
                        min: 1.0,
                        max: 20.0,
                    },
                ],
            },
            previous_slot: AtomicI32::new(-1),
            eating_slot: AtomicI32::new(-1),
        }
    }

    fn restore_slot(&self) -> anyhow::Result<()> {
        let previous = self.previous_slot.swap(-1, Ordering::SeqCst);
        let eating = self.eating_slot.swap(-1, Ordering::SeqCst);
        if previous < 0 {
            return Ok(());
        }
        let Some(player) = minecraft().player()? else {
            return Ok(());
        };
        let inventory = player.get_inventory()?;
        if inventory.selected()? == eating {
            inventory.set_selected(previous)?;
        }
        Ok(())
    }
}

impl Module for AutoEatModule {
    fn on_start(&self) -> anyhow::Result<()> {
        self.previous_slot.store(-1, Ordering::SeqCst);
        self.eating_slot.store(-1, Ordering::SeqCst);
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        self.restore_slot()
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let mc = minecraft();
        if mc.current_screen().is_open() {
            return Ok(());
        }
        let (Some(player), Some(game_mode)) = (mc.player()?, mc.game_mode()?) else {
            return Ok(());
        };

        let health_threshold = self
            .module
            .get_setting("Health Threshold")
            .and_then(|setting| setting.get_slider_value())
            .unwrap_or(10.0);
        let hunger_threshold = self
            .module
            .get_setting("Hunger Threshold")
            .and_then(|setting| setting.get_slider_value())
            .unwrap_or(16.0) as i32;
        let hunger = player.food_level()?;
        let should_eat =
            hunger <= hunger_threshold || (player.get_health()? <= health_threshold && hunger < 20);
        if !should_eat {
            self.restore_slot()?;
            return Ok(());
        }

        if player.is_using_item()? {
            return Ok(());
        }

        let inventory = player.get_inventory()?;
        let selected = inventory.selected()?;
        let mut food_slot = None;
        for slot in 0..9 {
            let stack = inventory.get_item(slot)?;
            if !stack.is_empty()? && stack.is_edible()? {
                food_slot = Some(slot);
                if slot == selected {
                    break;
                }
            }
        }
        let Some(food_slot) = food_slot else {
            return Ok(());
        };

        if self.previous_slot.load(Ordering::SeqCst) < 0 && selected != food_slot {
            self.previous_slot.store(selected, Ordering::SeqCst);
            self.eating_slot.store(food_slot, Ordering::SeqCst);
        }
        if selected != food_slot {
            inventory.set_selected(food_slot)?;
            self.eating_slot.store(food_slot, Ordering::SeqCst);
        }
        if let Err(error) = game_mode.use_main_hand(&player) {
            self.restore_slot()?;
            return Err(error);
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
