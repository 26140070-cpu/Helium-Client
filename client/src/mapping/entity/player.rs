use crate::mapping::client::player_info::PlayerInfo;
use crate::mapping::entity::{EntityRef, LivingEntityRef, PlayerRef};
use crate::mapping::inventory::{AbstractContainerMenu, EquipmentSlot, Inventory, ItemStack};
use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::{GlobalRef, JValue};
use jni::sys::jboolean;

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = Player)]
pub struct Player {
    jni_ref: GlobalRef,
}

impl Player {
    pub fn new(jni_ref: GlobalRef) -> Player {
        Player { jni_ref }
    }
}

impl EntityRef for Player {}
impl LivingEntityRef for Player {}
impl PlayerRef for Player {}

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = LocalPlayer)]
pub struct LocalPlayer {
    jni_ref: GlobalRef,
    pub abilities: Abilities,
}

impl LocalPlayer {
    pub fn new(player_ref: GlobalRef) -> anyhow::Result<Self> {
        Ok(Self {
            abilities: Abilities::new(player_ref.clone())?,
            jni_ref: player_ref,
        })
    }

    pub fn get_inventory(&self) -> anyhow::Result<Inventory> {
        mapping().in_frame(|| {
            let inventory = mapping()
                .call_method(
                    MinecraftClassType::Player,
                    self.jni_ref().as_obj(),
                    "getInventory",
                    &[],
                )?
                .l()?;
            Ok(Inventory::new(mapping().new_global_ref(inventory)?))
        })
    }

    pub fn food_level(&self) -> anyhow::Result<i32> {
        mapping().in_frame(|| {
            let food_data = mapping()
                .call_method(
                    MinecraftClassType::Player,
                    self.jni_ref().as_obj(),
                    "getFoodData",
                    &[],
                )?
                .l()?;
            Ok(mapping()
                .call_method(
                    MinecraftClassType::FoodData,
                    &food_data,
                    "getFoodLevel",
                    &[],
                )?
                .i()?)
        })
    }

    pub fn container_menu(&self) -> anyhow::Result<AbstractContainerMenu> {
        mapping().in_frame(|| {
            let menu = mapping()
                .get_field(
                    MinecraftClassType::Player,
                    self.jni_ref().as_obj(),
                    "containerMenu",
                    FieldType::Object(MinecraftClassType::AbstractContainerMenu),
                )?
                .l()?;
            Ok(AbstractContainerMenu::new(mapping().new_global_ref(menu)?))
        })
    }

    pub fn player_info(&self) -> anyhow::Result<PlayerInfo> {
        mapping().in_frame(|| {
            let info = mapping()
                .get_field(
                    MinecraftClassType::LocalPlayer,
                    self.jni_ref().as_obj(),
                    "playerInfo",
                    FieldType::Object(MinecraftClassType::PlayerInfo),
                )?
                .l()?;
            if info.is_null() {
                anyhow::bail!("LocalPlayer.playerInfo is null");
            }
            Ok(PlayerInfo::new(mapping().new_global_ref(info)?))
        })
    }

    pub fn forward_packet(&self, packet: &GlobalRef) -> anyhow::Result<()> {
        mapping().in_frame(|| {
            let mut env = mapping().get_env()?;
            let listener = env
                .get_field(
                    self.jni_ref().as_obj(),
                    "connection",
                    "Lnet/minecraft/client/multiplayer/ClientPacketListener;",
                )?
                .l()?;
            if listener.is_null() {
                return Ok(());
            }
            env.call_method(
                packet.as_obj(),
                "handle",
                "(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
                &[JValue::Object(&listener)],
            )?;
            Ok(())
        })
    }

    pub fn equipment_slot_for_item(&self, stack: &ItemStack) -> anyhow::Result<EquipmentSlot> {
        mapping().in_frame(|| {
            let slot = mapping()
                .call_method(
                    MinecraftClassType::LivingEntity,
                    self.jni_ref().as_obj(),
                    "getEquipmentSlotForItem",
                    &[JValue::Object(stack.jni_ref().as_obj())],
                )?
                .l()?;
            Ok(EquipmentSlot::new(mapping().new_global_ref(slot)?))
        })
    }
}

impl EntityRef for LocalPlayer {}
impl LivingEntityRef for LocalPlayer {}
impl PlayerRef for LocalPlayer {}

#[derive(Debug, Clone, MappedObject)]
#[mapped(class = Abilities)]
pub struct Abilities {
    jni_ref: GlobalRef,
}

impl Abilities {
    pub fn new(player: GlobalRef) -> anyhow::Result<Self> {
        mapping().in_frame(|| {
            let jni_ref = mapping()
                .call_method(MinecraftClassType::Player, &player, "getAbilities", &[])?
                .l()?;
            Ok(Self {
                jni_ref: mapping().new_global_ref(jni_ref)?,
            })
        })
    }

    pub fn fly(&self, value: bool) -> anyhow::Result<()> {
        let value: jboolean = if value { 1 } else { 0 };
        self.set_field("flying", FieldType::Boolean, JValue::Bool(value))?;
        self.set_field("mayfly", FieldType::Boolean, JValue::Bool(value))?;
        Ok(())
    }

    pub fn is_flying(&self) -> anyhow::Result<bool> {
        Ok(self.get_field("flying", FieldType::Boolean)?.z()?)
    }

    pub fn set_flying_speed(&self, speed: f32) -> anyhow::Result<()> {
        self.call_method("setFlyingSpeed", &[JValue::Float(speed)])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_may_fly(&self) -> anyhow::Result<bool> {
        Ok(self.get_field("mayfly", FieldType::Boolean)?.z()?)
    }
}
