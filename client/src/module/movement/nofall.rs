use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId};
use crate::net::packet::{Packet, PacketAction};

#[derive(Debug)]
pub struct NoFallModule {
    pub module: ModuleData,
}

impl NoFallModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::NoFall,
                description: "Prevents fall damage".to_string(),
                category: ModuleCategory::Movement,
                key_bind: KeyboardKey::KeyN,
                enabled: false,
                settings: vec![],
            },
        }
    }
}

impl Module for NoFallModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_packet(&self, packet: &mut Packet) -> PacketAction {
        if let Packet::ServerboundMovePlayer(move_packet) = packet {
            move_packet.on_ground = true;
        }
        PacketAction::Forward
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
