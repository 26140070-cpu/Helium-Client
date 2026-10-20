use crate::module::{KeyboardKey, Module, ModuleCategory, ModuleData, ModuleId, ModuleSetting};
use crate::state::client;

#[derive(Debug)]
pub struct TerminalModule {
    pub module: ModuleData,
}

impl TerminalModule {
    pub fn new() -> Self {
        Self {
            module: ModuleData {
                id: ModuleId::Terminal,
                description: "Optional terminal docked to the top or bottom edge".to_string(),
                category: ModuleCategory::Client,
                key_bind: KeyboardKey::KeyNone,
                enabled: false,
                settings: vec![
                    ModuleSetting::Toggle {
                        name: "AutoScroll".to_string(),
                        value: true,
                    },
                    ModuleSetting::Choice {
                        name: "Position".to_string(),
                        value: 0,
                        options: vec!["Bottom".to_string(), "Top".to_string()],
                    },
                    ModuleSetting::Slider {
                        name: "MaxLines".to_string(),
                        value: 300.0,
                        min: 50.0,
                        max: 500.0,
                    },
                ],
            },
        }
    }

    pub fn is_active() -> bool {
        client()
            .modules
            .get(ModuleId::Terminal)
            .and_then(|arc| arc.lock().ok().map(|m| m.get_module_data().enabled))
            .unwrap_or(false)
    }

    pub fn is_top() -> bool {
        client()
            .modules
            .get(ModuleId::Terminal)
            .and_then(|arc| {
                arc.lock().ok().and_then(|module| {
                    module
                        .get_module_data()
                        .get_setting("Position")
                        .and_then(|setting| match setting {
                            ModuleSetting::Choice { value, .. } => Some(*value == 1),
                            _ => None,
                        })
                })
            })
            .unwrap_or(false)
    }

    pub fn toggle_position() {
        let Some(module) = client().modules.get(ModuleId::Terminal) else {
            log::warn!("terminal: cannot change dock position because the module is missing");
            return;
        };
        let Ok(mut module) = module.lock() else {
            log::error!("terminal: module lock poisoned while changing dock position");
            return;
        };
        let Some(ModuleSetting::Choice { value, options, .. }) =
            module.get_module_data_mut().get_setting_mut("Position")
        else {
            log::error!("terminal: Position setting is missing or has an invalid type");
            return;
        };
        if options.len() < 2 {
            log::error!("terminal: Position setting must contain Top and Bottom choices");
            return;
        }
        *value = if *value == 0 { 1 } else { 0 };
        drop(module);
        crate::config::save();
    }
}

impl Module for TerminalModule {
    fn on_start(&self) -> anyhow::Result<()> {
        crate::graphic::input::set_terminal_cursor(true);
        crate::log_buffer::push("INFO", "Terminal opened");
        Ok(())
    }
    fn on_stop(&self) -> anyhow::Result<()> {
        crate::graphic::input::set_terminal_cursor(false);
        crate::log_buffer::push("INFO", "Terminal closed");
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
