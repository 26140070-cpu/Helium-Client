use crate::mapping::client::game_renderer::GameRenderer;
use crate::mapping::client::gamemode::MultiPlayerGameMode;
use crate::mapping::client::options::Options;
use crate::mapping::client::screen::Screen;
use crate::mapping::client::window::Window;
use crate::mapping::client::world::World;
use crate::mapping::entity::player::LocalPlayer;
use crate::mapping::{FieldType, MappedObject, MinecraftClassType};
use crate::state::mapping;
use jni::objects::GlobalRef;
use std::sync::RwLock;

#[derive(Debug, MappedObject)]
#[mapped(class = Minecraft)]
pub struct Minecraft {
    jni_ref: GlobalRef,
    pub window: Window,

    player: RwLock<Option<LocalPlayer>>,
}

impl Minecraft {
    pub fn new() -> anyhow::Result<Minecraft> {
        let jni_ref = mapping().in_frame(|| {
            let minecraft = mapping()
                .call_static_method(MinecraftClassType::Minecraft, "getInstance", &[])?
                .l()?;
            if minecraft.is_null() {
                return Err(anyhow::anyhow!("Minecraft.getInstance() returned null"));
            }
            mapping().new_global_ref(minecraft)
        })?;
        let window = Window::new(&jni_ref)?;

        Ok(Minecraft {
            jni_ref,
            window,
            player: RwLock::new(None),
        })
    }

    pub fn player(&self) -> anyhow::Result<Option<LocalPlayer>> {
        let Some(player_ref) = self.world_object("player", MinecraftClassType::LocalPlayer)? else {
            if let Ok(mut cache) = self.player.write() {
                *cache = None;
            }
            return Ok(None);
        };

        {
            let cache = self
                .player
                .read()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            if let Some(cached) = cache.as_ref() {
                if mapping()
                    .get_env()?
                    .is_same_object(cached.jni_ref(), &player_ref)?
                {
                    return Ok(Some(cached.clone()));
                }
            }
        }

        let player = LocalPlayer::new(player_ref)?;
        *self
            .player
            .write()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))? = Some(player.clone());
        Ok(Some(player))
    }

    pub fn world(&self) -> anyhow::Result<Option<World>> {
        Ok(self
            .world_object("level", MinecraftClassType::Level)?
            .map(World::new))
    }

    pub fn game_mode(&self) -> anyhow::Result<Option<MultiPlayerGameMode>> {
        Ok(self
            .world_object("gameMode", MinecraftClassType::MultiPlayerGameMode)?
            .map(MultiPlayerGameMode::new))
    }

    pub fn game_renderer(&self) -> anyhow::Result<GameRenderer> {
        self.in_frame(|| {
            let obj = self
                .get_field(
                    "gameRenderer",
                    FieldType::Object(MinecraftClassType::GameRenderer),
                )?
                .l()?;
            Ok(GameRenderer::new(mapping().new_global_ref(obj)?))
        })
    }

    pub fn options(&self) -> anyhow::Result<Options> {
        self.in_frame(|| {
            let obj = self
                .get_field("options", FieldType::Object(MinecraftClassType::Options))?
                .l()?;
            Ok(Options::new(mapping().new_global_ref(obj)?))
        })
    }

    pub fn in_world(&self) -> bool {
        matches!(self.player(), Ok(Some(_)))
    }

    pub fn teardown(&self) {
        if let Ok(mut cache) = self.player.write() {
            *cache = None;
        }
    }

    pub fn current_screen_is_null(&self) -> bool {
        self.current_screen() == Screen::None
    }

    pub fn current_screen(&self) -> Screen {
        self.in_frame(|| {
            let screen = self
                .get_field("screen", FieldType::Object(MinecraftClassType::Screen))?
                .l()?;
            if screen.is_null() {
                return Ok(Screen::None);
            }

            let is = |class| mapping().is_instance_of(class, &screen).unwrap_or(false);

            let screen = if is(MinecraftClassType::ChatScreen) {
                Screen::Chat
            } else if is(MinecraftClassType::CreativeModeInventoryScreen)
                || is(MinecraftClassType::InventoryScreen)
            {
                Screen::Inventory
            } else if is(MinecraftClassType::CraftingScreen) {
                Screen::Crafting
            } else if is(MinecraftClassType::AbstractContainerScreen) {
                Screen::Container
            } else if is(MinecraftClassType::PauseScreen) {
                Screen::Menu
            } else {
                Screen::Unknown
            };
            Ok(screen)
        })
        .unwrap_or(Screen::Unknown)
    }

    fn world_object(
        &self,
        field: &str,
        class: MinecraftClassType,
    ) -> anyhow::Result<Option<GlobalRef>> {
        self.in_frame(|| {
            let obj = self.get_field(field, FieldType::Object(class))?.l()?;
            if obj.is_null() {
                return Ok(None);
            }
            Ok(Some(mapping().new_global_ref(obj)?))
        })
    }
}
