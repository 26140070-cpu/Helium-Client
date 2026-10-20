use crate::mapping::client::player_info::game_type_spectator;
use crate::mapping::entity::EntityRef;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::net::packet::{Packet, PacketAction};
use crate::state::minecraft;
use jni::objects::GlobalRef;
use std::sync::Mutex;

#[derive(Debug)]
struct FreecamState {
    anchor: (f64, f64, f64),

    anchor_yaw: f32,
    anchor_pitch: f32,

    previous_game_mode: GlobalRef,

    previous_flying: bool,

    queued_info_updates: Vec<GlobalRef>,
}

#[derive(Debug)]
pub struct FreecamModule {
    pub module: ModuleData,
    state: Mutex<Option<FreecamState>>,
}

impl FreecamModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Freecam,
                description: "Flies the view freely while the body stays put".to_string(),
                category: ModuleCategory::Player,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
            state: Mutex::new(None),
        }
    }
}

impl Module for FreecamModule {
    fn on_start(&self) -> anyhow::Result<()> {
        let Some(player) = minecraft().player()? else {
            return Ok(());
        };
        let feet = player.get_position()?;
        let anchor_yaw = player.get_yaw()?;
        let anchor_pitch = player.get_pitch()?;
        let player_info = player.player_info()?;
        let previous_game_mode = player_info.get_game_mode()?;
        let previous_flying = player.abilities.is_flying()?;

        let spectator = game_type_spectator()?;
        player_info.set_game_mode(&spectator)?;
        player.abilities.fly(true)?;

        *self.state.lock().unwrap() = Some(FreecamState {
            anchor: (feet.x(), feet.y(), feet.z()),
            anchor_yaw,
            anchor_pitch,
            previous_game_mode,
            previous_flying,
            queued_info_updates: Vec::new(),
        });
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        let Some(state) = self.state.lock().unwrap().take() else {
            return Ok(());
        };
        if let Some(player) = minecraft().player()? {
            if let Ok(info) = player.player_info() {
                let _ = info.set_game_mode(&state.previous_game_mode);
            }
            player.abilities.fly(state.previous_flying)?;
            player.set_pos(state.anchor.0, state.anchor.1, state.anchor.2)?;
            player.set_rotation(state.anchor_yaw, state.anchor_pitch)?;
            player.set_delta_movement(0.0, 0.0, 0.0)?;

            for packet in &state.queued_info_updates {
                if let Err(error) = player.forward_packet(packet) {
                    log::warn!("Freecam: replay of a queued packet failed: {error}");
                }
            }
        }
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_packet(&self, packet: &mut Packet) -> PacketAction {
        match packet {
            Packet::ServerboundMovePlayer(_) => PacketAction::Cancel,

            Packet::ClientboundPlayerInfoUpdate(captured) => {
                if let Ok(mut guard) = self.state.lock() {
                    if let Some(state) = guard.as_mut() {
                        state.queued_info_updates.push(captured.jni_ref.clone());
                        return PacketAction::Cancel;
                    }
                }
                PacketAction::Forward
            }
            _ => PacketAction::Forward,
        }
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
