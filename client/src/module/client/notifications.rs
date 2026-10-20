use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::client;

#[derive(Debug)]
pub struct NotificationsModule {
    pub module: ModuleData,
}

impl NotificationsModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Notifications,
                description: "Toggle and style client notifications".to_string(),
                category: ModuleCategory::Client,
                key_bind: KeyboardKey::KeyNone,
                enabled: true,
                settings: vec![
                    ModuleSetting::Toggle {
                        name: "ModuleToggle".to_string(),
                        value: true,
                    },
                    ModuleSetting::Toggle {
                        name: "Warnings".to_string(),
                        value: true,
                    },
                    ModuleSetting::Slider {
                        name: "Duration".to_string(),
                        value: 3.0,
                        min: 1.0,
                        max: 8.0,
                    },
                    ModuleSetting::Choice {
                        name: "Position".to_string(),
                        value: 0,
                        options: vec![
                            "TopRight".to_string(),
                            "BottomRight".to_string(),
                            "TopLeft".to_string(),
                        ],
                    },
                ],
            },
        }
    }

    pub fn show_module_toggle() -> bool {
        client()
            .modules
            .get(ModuleId::Notifications)
            .and_then(|arc| {
                let m = arc.lock().ok()?;
                if !m.get_module_data().enabled {
                    return Some(false);
                }
                m.get_module_data()
                    .get_setting("ModuleToggle")
                    .and_then(|s| s.get_toggle_value())
            })
            .unwrap_or(true)
    }

    pub fn duration() -> f32 {
        client()
            .modules
            .get(ModuleId::Notifications)
            .and_then(|arc| {
                let m = arc.lock().ok()?;
                m.get_module_data()
                    .get_setting("Duration")
                    .and_then(|s| s.get_slider_value())
            })
            .unwrap_or(3.0)
    }
}

impl Module for NotificationsModule {
    fn on_start(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn on_stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn on_tick(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_module_data(&self) -> &ModuleData {
        &self.module
    }
    fn get_module_data_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
