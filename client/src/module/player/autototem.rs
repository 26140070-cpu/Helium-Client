use crate::mapping::inventory::{self, Inventory, Item};
use crate::mapping::MappedObject;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::state::minecraft;
use std::sync::atomic::{AtomicI32, Ordering};

const INVENTORY_MENU_ID: i32 = 0;

const OFFHAND_SWAP_BUTTON: i32 = Inventory::SLOT_OFFHAND;

const STORAGE_SLOTS: i32 = 36;

#[derive(Debug)]
pub struct AutoTotemModule {
    pub module: ModuleData,
    swap_cooldown: AtomicI32,
}

impl AutoTotemModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::AutoTotem,
                description: "Keeps a Totem of Undying in the off-hand".to_string(),
                category: ModuleCategory::Player,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
            swap_cooldown: AtomicI32::new(0),
        }
    }
}

impl Module for AutoTotemModule {
    fn on_start(&self) -> anyhow::Result<()> {
        self.swap_cooldown.store(0, Ordering::SeqCst);
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let Some(player) = minecraft().player()? else {
            return Ok(());
        };

        let cooldown = self.swap_cooldown.load(Ordering::SeqCst);
        if cooldown > 0 {
            self.swap_cooldown.store(cooldown - 1, Ordering::SeqCst);
            return Ok(());
        }

        if player.container_menu()?.container_id()? != INVENTORY_MENU_ID {
            return Ok(());
        }

        let inventory = player.get_inventory()?;
        let totem = inventory::totem_of_undying()?;

        if slot_holds(&inventory, Inventory::SLOT_OFFHAND, &totem)? {
            return Ok(());
        }

        let Some(inventory_slot) = find_totem(&inventory, &totem)? else {
            return Ok(());
        };

        let Some(game_mode) = minecraft().game_mode()? else {
            return Ok(());
        };
        game_mode.container_swap(
            INVENTORY_MENU_ID,
            Inventory::menu_slot(inventory_slot),
            OFFHAND_SWAP_BUTTON,
            &player,
        )?;
        self.swap_cooldown.store(2, Ordering::SeqCst);
        Ok(())
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}

fn slot_holds(inventory: &Inventory, slot: i32, item: &Item) -> anyhow::Result<bool> {
    let stack = inventory.get_item(slot)?;
    if stack.is_empty()? {
        return Ok(false);
    }
    Ok(stack.get_item()?.is_same(item))
}

fn find_totem(inventory: &Inventory, totem: &Item) -> anyhow::Result<Option<i32>> {
    for slot in 0..STORAGE_SLOTS {
        if slot_holds(inventory, slot, totem)? {
            return Ok(Some(slot));
        }
    }
    Ok(None)
}
