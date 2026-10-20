use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::net::packet::serverbound_move_player::ServerboundMovePlayerPacket;
use crate::net::packet::{Packet, PacketAction};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const MAX_BUFFERED_MOVEMENT_PACKETS: usize = 600;

#[derive(Debug)]
pub struct BlinkModule {
    pub module: ModuleData,
    packets: Mutex<VecDeque<ServerboundMovePlayerPacket>>,
    saturated: AtomicBool,
}

impl BlinkModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Blink,
                description: "Buffers outgoing movement packets until disabled".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![],
            },
            packets: Mutex::new(VecDeque::new()),
            saturated: AtomicBool::new(false),
        }
    }
}

impl Module for BlinkModule {
    fn on_start(&self) -> anyhow::Result<()> {
        self.packets
            .lock()
            .map_err(|_| anyhow::anyhow!("Blink packet queue lock is poisoned"))?
            .clear();
        self.saturated.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        let packets = {
            let mut queue = self
                .packets
                .lock()
                .map_err(|_| anyhow::anyhow!("Blink packet queue lock is poisoned"))?;
            queue.drain(..).collect::<Vec<_>>()
        };

        self.saturated.store(false, Ordering::SeqCst);
        let mut first_error = None;
        for (index, packet) in packets.iter().copied().enumerate() {
            if let Err(error) = crate::net::send_movement_packet(packet) {
                log::error!(
                    "Blink: could not flush buffered movement packet {}/{}: {error}",
                    index + 1,
                    packets.len()
                );
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_packet(&self, packet: &mut Packet) -> PacketAction {
        let Packet::ServerboundMovePlayer(packet) = packet else {
            return PacketAction::Forward;
        };
        let Ok(mut queue) = self.packets.lock() else {
            log::error!("Blink: packet queue lock is poisoned; forwarding movement");
            return PacketAction::Forward;
        };
        if queue.len() >= MAX_BUFFERED_MOVEMENT_PACKETS {
            if !self.saturated.swap(true, Ordering::SeqCst) {
                log::error!(
                    "Blink: movement buffer reached its limit; pausing outgoing movement until disabled"
                );
            }
            return PacketAction::Cancel;
        }
        queue.push_back(*packet);
        PacketAction::Cancel
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_BUFFERED_MOVEMENT_PACKETS, *};

    #[test]
    fn blink_buffer_is_bounded_and_cancels_when_full() {
        let module = BlinkModule::new();
        let mut packet = Packet::ServerboundMovePlayer(ServerboundMovePlayerPacket {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            y_rot: 0.0,
            x_rot: 0.0,
            on_ground: true,
            horizontal_collision: false,
            has_position: true,
            has_rotation: false,
        });
        for _ in 0..MAX_BUFFERED_MOVEMENT_PACKETS {
            assert_eq!(module.handle_packet(&mut packet), PacketAction::Cancel);
        }
        assert_eq!(module.handle_packet(&mut packet), PacketAction::Cancel);
        assert_eq!(
            module.packets.lock().expect("queue lock").len(),
            MAX_BUFFERED_MOVEMENT_PACKETS
        );
    }
}
