use crate::mapping::java::iterable::Iterable;
use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JValue};

#[derive(Debug, MappedObject)]
#[mapped(class = Inventory)]
pub struct Inventory {
    jni_ref: GlobalRef,
}

impl Inventory {
    pub const SLOT_OFFHAND: i32 = 40;

    pub fn new(jni_ref: GlobalRef) -> Inventory {
        Inventory { jni_ref }
    }

    pub fn menu_slot(inventory_slot: i32) -> i32 {
        if inventory_slot < 9 {
            inventory_slot + 36
        } else {
            inventory_slot
        }
    }

    pub fn get_item(&self, slot: i32) -> anyhow::Result<ItemStack> {
        self.in_frame(|| {
            let stack = self.call_method("getItem", &[JValue::Int(slot)])?.l()?;
            Ok(ItemStack::new(mapping().new_global_ref(stack)?))
        })
    }
}

#[derive(Debug, MappedObject)]
#[mapped(class = ItemStack)]
pub struct ItemStack {
    jni_ref: GlobalRef,
}

impl ItemStack {
    pub fn new(jni_ref: GlobalRef) -> ItemStack {
        ItemStack { jni_ref }
    }

    pub fn is_empty(&self) -> anyhow::Result<bool> {
        Ok(self.call_method("isEmpty", &[])?.z()?)
    }

    pub fn get_item(&self) -> anyhow::Result<Item> {
        self.in_frame(|| {
            let item = self.call_method("getItem", &[])?.l()?;
            Ok(Item::new(mapping().new_global_ref(item)?))
        })
    }

    pub fn is_edible(&self) -> anyhow::Result<bool> {
        Ok(self.call_method("isEdible", &[])?.z()?)
    }
}

#[derive(Debug, MappedObject)]
#[mapped(class = Item)]
pub struct Item {
    jni_ref: GlobalRef,
}

impl Item {
    pub fn new(jni_ref: GlobalRef) -> Item {
        Item { jni_ref }
    }
}

#[derive(Debug, MappedObject)]
#[mapped(class = AbstractContainerMenu)]
pub struct AbstractContainerMenu {
    jni_ref: GlobalRef,
}

impl AbstractContainerMenu {
    pub fn new(jni_ref: GlobalRef) -> AbstractContainerMenu {
        AbstractContainerMenu { jni_ref }
    }

    pub fn container_id(&self) -> anyhow::Result<i32> {
        Ok(self.get_field("containerId", FieldType::Int)?.i()?)
    }

    pub fn get_items(&self) -> anyhow::Result<Vec<ItemStack>> {
        self.in_frame(|| {
            let list = self.call_method("getItems", &[])?.l()?;
            let iterable = Iterable::new(mapping().new_global_ref(list)?);
            let iterator = iterable.iterator()?;
            let mut items = Vec::new();
            while iterator.has_next()? {
                items.push(ItemStack::new(iterator.next()?));
            }
            Ok(items)
        })
    }
}

#[derive(Debug, MappedObject)]
#[mapped(class = EquipmentSlot)]
pub struct EquipmentSlot {
    jni_ref: GlobalRef,
}

impl EquipmentSlot {
    pub fn new(jni_ref: GlobalRef) -> EquipmentSlot {
        EquipmentSlot { jni_ref }
    }

    pub fn is_armor(&self) -> anyhow::Result<bool> {
        Ok(self.call_method("isArmor", &[])?.z()?)
    }

    pub fn index(&self, base: i32) -> anyhow::Result<i32> {
        Ok(self.call_method("getIndex", &[JValue::Int(base)])?.i()?)
    }
}

pub fn totem_of_undying() -> anyhow::Result<Item> {
    mapping().in_frame(|| {
        let item = mapping()
            .get_static_field(
                MinecraftClassType::Items,
                "TOTEM_OF_UNDYING",
                FieldType::Object(MinecraftClassType::Item),
            )?
            .l()?;
        Ok(Item::new(mapping().new_global_ref(item)?))
    })
}

pub fn end_crystal() -> anyhow::Result<Item> {
    mapping().in_frame(|| {
        let item = mapping()
            .get_static_field(
                MinecraftClassType::Items,
                "END_CRYSTAL",
                FieldType::Object(MinecraftClassType::Item),
            )?
            .l()?;
        Ok(Item::new(mapping().new_global_ref(item)?))
    })
}

pub fn obsidian() -> anyhow::Result<Item> {
    mapping().in_frame(|| {
        let item = mapping()
            .get_static_field(
                MinecraftClassType::Items,
                "OBSIDIAN",
                FieldType::Object(MinecraftClassType::Item),
            )?
            .l()?;
        Ok(Item::new(mapping().new_global_ref(item)?))
    })
}

pub fn glowstone() -> anyhow::Result<Item> {
    mapping().in_frame(|| {
        let item = mapping()
            .get_static_field(
                MinecraftClassType::Items,
                "GLOWSTONE",
                FieldType::Object(MinecraftClassType::Item),
            )?
            .l()?;
        Ok(Item::new(mapping().new_global_ref(item)?))
    })
}

pub fn respawn_anchor() -> anyhow::Result<Item> {
    mapping().in_frame(|| {
        let item = mapping()
            .get_static_field(
                MinecraftClassType::Items,
                "RESPAWN_ANCHOR",
                FieldType::Object(MinecraftClassType::Item),
            )?
            .l()?;
        Ok(Item::new(mapping().new_global_ref(item)?))
    })
}

impl Inventory {
    pub fn selected(&self) -> anyhow::Result<i32> {
        Ok(self.get_field("selected", FieldType::Int)?.i()?)
    }

    pub fn set_selected(&self, slot: i32) -> anyhow::Result<()> {
        self.set_field("selected", FieldType::Int, JValue::Int(slot))
    }

    pub fn find_hotbar(&self, item: &Item) -> anyhow::Result<Option<i32>> {
        for slot in 0..9 {
            let stack = self.get_item(slot)?;
            if stack.is_empty()? {
                continue;
            }
            if stack.get_item()?.is_same(item) {
                return Ok(Some(slot));
            }
        }
        Ok(None)
    }

    pub fn find_item(&self, item: &Item) -> anyhow::Result<Option<i32>> {
        for slot in 0..36 {
            let stack = self.get_item(slot)?;
            if stack.is_empty()? {
                continue;
            }
            if stack.get_item()?.is_same(item) {
                return Ok(Some(slot));
            }
        }
        Ok(None)
    }
}
