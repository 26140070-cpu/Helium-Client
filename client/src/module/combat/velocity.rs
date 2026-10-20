use crate::mapping::entity::EntityRef;
use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::net::packet::{Packet, PacketAction};
use crate::state::minecraft;
use std::sync::atomic::{AtomicI32, Ordering};

const UNKNOWN_ID: i32 = i32::MIN;

#[derive(Debug)]
pub struct VelocityModule {
    pub module: ModuleData,

    local_id: AtomicI32,
}

impl VelocityModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Velocity,
                description: "Reduces or cancels server knockback".to_string(),
                category: ModuleCategory::Combat,
                key_bind: KeyboardKey::KeyV,
                enabled: false,
                settings: vec![
                    ModuleSetting::Slider {
                        name: "Horizontal".to_string(),
                        value: 0.0,
                        min: 0.0,
                        max: 100.0,
                    },
                    ModuleSetting::Slider {
                        name: "Vertical".to_string(),
                        value: 0.0,
                        min: 0.0,
                        max: 100.0,
                    },
                ],
            },
            local_id: AtomicI32::new(UNKNOWN_ID),
        }
    }

    fn factor(&self, name: &str) -> f64 {
        self.module
            .get_setting(name)
            .and_then(|setting| setting.get_slider_value())
            .unwrap_or(0.0) as f64
            / 100.0
    }
}

impl Module for VelocityModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_stop(&self) -> anyhow::Result<()> {
        self.local_id.store(UNKNOWN_ID, Ordering::Relaxed);
        Ok(())
    }

    fn on_tick(&self) -> anyhow::Result<()> {
        let id = match minecraft().player()? {
            Some(player) => player.id()?,
            None => UNKNOWN_ID,
        };
        self.local_id.store(id, Ordering::Relaxed);
        Ok(())
    }

    fn handle_packet(&self, packet: &mut Packet) -> PacketAction {
        let Packet::ClientboundSetEntityMotion(motion) = packet else {
            return PacketAction::Forward;
        };

        if motion.entity_id != self.local_id.load(Ordering::Relaxed) {
            return PacketAction::Forward;
        }
        let horizontal = self.factor("Horizontal");
        let vertical = self.factor("Vertical");
        motion.x *= horizontal;
        motion.y *= vertical;
        motion.z *= horizontal;
        PacketAction::Forward
    }

    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }

    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
