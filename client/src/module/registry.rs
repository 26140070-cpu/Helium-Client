use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use log::error;

use crate::module::{KeyboardKey, Module, ModuleId, ModuleSetting, ModuleType};
use crate::net::packet::{Packet, PacketAction};

pub type ModuleHandle = Arc<Mutex<ModuleType>>;

struct ModuleDefaults {
    key_bind: KeyboardKey,
    settings: Vec<ModuleSetting>,
}

#[derive(Default)]
pub struct ModuleRegistry {
    modules: DashMap<ModuleId, ModuleHandle>,

    defaults: DashMap<ModuleId, ModuleDefaults>,

    active: AtomicBool,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<M>(&self, module: M)
    where
        M: Module + Send + Sync + 'static,
    {
        let module: ModuleType = Box::new(module);
        let data = module.get_module_data();
        let id = data.id;
        self.defaults.insert(
            id,
            ModuleDefaults {
                key_bind: data.key_bind,
                settings: data.settings.clone(),
            },
        );
        self.modules.insert(id, Arc::new(Mutex::new(module)));
    }

    /// Restores every module to its factory defaults: default keybind, default
    /// setting values, and disabled. Backs the GUI's "Reset Settings" button.
    pub fn reset_settings(&self) {
        for entry in self.modules.iter() {
            let Some(defaults) = self.defaults.get(entry.key()) else {
                continue;
            };
            let Ok(mut module) = entry.value().lock() else {
                continue;
            };
            if module.get_module_data().enabled && self.is_active() {
                let _ = module.on_stop();
            }
            let data = module.get_module_data_mut();
            data.key_bind = defaults.key_bind;
            data.settings = defaults.settings.clone();
            data.enabled = false;
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn set_active(&self, active: bool) {
        if self.active.swap(active, Ordering::SeqCst) == active {
            return;
        }
        for handle in self.handles() {
            let Ok(module) = handle.lock() else {
                continue;
            };
            if !module.get_module_data().enabled {
                continue;
            }
            let result = if active {
                module.on_start()
            } else {
                module.on_stop()
            };
            if let Err(e) = result {
                let name = module.get_module_data().name();
                let verb = if active { "start" } else { "stop" };
                error!("module '{name}' failed to {verb} on world transition: {e}");
            }
        }
    }

    pub fn get(&self, id: ModuleId) -> Option<ModuleHandle> {
        self.modules.get(&id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn handles(&self) -> Vec<ModuleHandle> {
        self.modules
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect()
    }

    pub fn by_id(&self) -> HashMap<ModuleId, ModuleHandle> {
        self.modules
            .iter()
            .map(|entry| (*entry.key(), Arc::clone(entry.value())))
            .collect()
    }

    pub fn tick(&self) {
        for handle in self.handles() {
            let Ok(module) = handle.lock() else {
                continue;
            };
            if !module.get_module_data().enabled {
                continue;
            }
            if let Err(e) = module.on_tick() {
                let name = module.get_module_data().name();
                error!("module '{name}' tick failed, stopping it: {e}");
                if let Err(e) = module.on_stop() {
                    error!("module '{name}' also failed to stop: {e}");
                }
            }
        }
    }

    pub fn handle_packet(&self, packet: &mut Packet) -> PacketAction {
        if !self.is_active() {
            return PacketAction::Forward;
        }
        for handle in self.handles() {
            let Ok(module) = handle.lock() else {
                continue;
            };
            if !module.get_module_data().enabled {
                continue;
            }
            if module.handle_packet(packet) == PacketAction::Cancel {
                return PacketAction::Cancel;
            }
        }
        PacketAction::Forward
    }
}
