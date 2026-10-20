use crate::module::{KeyboardKey, ModuleCategory, ModuleId, ModuleSetting};
use crate::state::client;
use anyhow::{Context, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::LazyLock;

const DEFAULT_GUI_OPEN_KEY: i32 = 344;
static GUI_OPEN_KEY: AtomicI32 = AtomicI32::new(DEFAULT_GUI_OPEN_KEY);

pub fn gui_open_key() -> i32 {
    let key = GUI_OPEN_KEY.load(Ordering::Relaxed);
    if key < 0 {
        DEFAULT_GUI_OPEN_KEY
    } else {
        key
    }
}

pub fn set_gui_open_key(key: i32) {
    GUI_OPEN_KEY.store(
        if key < 0 { DEFAULT_GUI_OPEN_KEY } else { key },
        Ordering::Relaxed,
    );
}

static PANEL_POS: LazyLock<DashMap<ModuleCategory, [f32; 2]>> = LazyLock::new(DashMap::new);
static HUD_POS: LazyLock<DashMap<String, [f32; 2]>> = LazyLock::new(DashMap::new);

static EXPANDED: LazyLock<DashMap<ModuleId, bool>> = LazyLock::new(DashMap::new);
static FAVORITES: LazyLock<DashMap<ModuleId, bool>> = LazyLock::new(DashMap::new);

pub fn panel_pos(category: ModuleCategory) -> Option<[f32; 2]> {
    PANEL_POS.get(&category).map(|entry| *entry)
}

pub fn set_panel_pos(category: ModuleCategory, pos: [f32; 2]) {
    PANEL_POS.insert(category, pos);
}

pub fn hud_pos(id: &str) -> Option<[f32; 2]> {
    HUD_POS.get(id).map(|entry| *entry)
}

pub fn set_hud_pos(id: &str, pos: [f32; 2]) {
    HUD_POS.insert(id.to_string(), pos);
}

pub fn is_expanded(id: ModuleId) -> bool {
    EXPANDED.get(&id).map(|entry| *entry).unwrap_or(false)
}

pub fn set_expanded(id: ModuleId, expanded: bool) {
    EXPANDED.insert(id, expanded);
}

pub fn is_favorite(id: ModuleId) -> bool {
    FAVORITES.get(&id).map(|entry| *entry).unwrap_or(false)
}

pub fn set_favorite(id: ModuleId, favorite: bool) {
    if favorite {
        FAVORITES.insert(id, true);
    } else {
        FAVORITES.remove(&id);
    }
}

pub fn favorites() -> HashSet<ModuleId> {
    FAVORITES.iter().map(|entry| *entry.key()).collect()
}

pub fn reset_ui_state() {
    PANEL_POS.clear();
    HUD_POS.clear();
    EXPANDED.clear();
}

const CONFIG_FILE: &str = "helium_client_config.json";
const PROFILES_DIR: &str = "profiles";
const MAX_PROFILE_NAME_LENGTH: usize = 64;

pub fn base_dir() -> PathBuf {
    match std::env::var_os("HELIUM_CONFIG_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from("."),
    }
}

fn config_path() -> PathBuf {
    base_dir().join(CONFIG_FILE)
}

fn validate_profile_name(name: &str) -> Result<()> {
    anyhow::ensure!(
        !name.is_empty()
            && name.len() <= MAX_PROFILE_NAME_LENGTH
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'),
        "invalid profile name '{name}': use 1-64 ASCII letters, numbers, '-' or '_'"
    );
    Ok(())
}

fn profile_path(name: &str) -> Result<PathBuf> {
    validate_profile_name(name)?;
    Ok(base_dir().join(PROFILES_DIR).join(format!("{name}.json")))
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    modules: Vec<ModuleConfig>,
    #[serde(default = "default_gui_open_key")]
    gui_open_key: i32,
    #[serde(default)]
    theme_preset: usize,
    #[serde(default)]
    custom_themes: Vec<crate::graphic::theme::CustomTheme>,
    #[serde(default)]
    active_custom_theme: Option<String>,
    #[serde(default)]
    favorites: Vec<ModuleId>,

    #[serde(default)]
    panels: Vec<PanelConfig>,
    #[serde(default)]
    hud_elements: Vec<HudConfig>,
}

fn default_gui_open_key() -> i32 {
    DEFAULT_GUI_OPEN_KEY
}

#[derive(Serialize, Deserialize)]
struct ModuleConfig {
    id: ModuleId,
    key_bind: i32,
    enabled: bool,

    #[serde(default)]
    expanded: bool,
    settings: Vec<SavedSetting>,
}

#[derive(Serialize, Deserialize)]
struct PanelConfig {
    category: ModuleCategory,
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize)]
struct HudConfig {
    id: String,
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize)]
struct SavedSetting {
    name: String,
    value: SettingValue,
}

#[derive(Serialize, Deserialize)]
enum SettingValue {
    Toggle(bool),
    Slider(f32),
    Choice(usize),
    Color([f32; 4]),
}

impl SettingValue {
    fn capture(setting: &ModuleSetting) -> SettingValue {
        match setting {
            ModuleSetting::Toggle { value, .. } => SettingValue::Toggle(*value),
            ModuleSetting::Slider { value, .. } => SettingValue::Slider(*value),
            ModuleSetting::Choice { value, .. } => SettingValue::Choice(*value),
            ModuleSetting::Color { value, .. } => SettingValue::Color(*value),
        }
    }

    fn apply(&self, setting: &mut ModuleSetting) {
        match (self, setting) {
            (SettingValue::Toggle(v), ModuleSetting::Toggle { value, .. }) => *value = *v,
            (
                SettingValue::Slider(v),
                ModuleSetting::Slider {
                    value, min, max, ..
                },
            ) => *value = v.clamp(*min, *max),
            (SettingValue::Choice(v), ModuleSetting::Choice { value, options, .. })
                if *v < options.len() =>
            {
                *value = *v;
            }
            (SettingValue::Color(v), ModuleSetting::Color { value, .. }) => *value = *v,
            _ => {}
        }
    }
}

fn capture_config() -> Config {
    let mut config = Config {
        gui_open_key: gui_open_key(),
        theme_preset: crate::graphic::theme::selected_preset(),
        custom_themes: crate::graphic::theme::custom_themes(),
        active_custom_theme: crate::graphic::theme::selected_custom_theme(),
        favorites: favorites().into_iter().collect(),
        ..Config::default()
    };
    for handle in client().modules.handles() {
        let Ok(module) = handle.lock() else {
            continue;
        };
        let data = module.get_module_data();
        config.modules.push(ModuleConfig {
            id: data.id,
            key_bind: data.key_bind as i32,
            enabled: data.enabled,
            expanded: is_expanded(data.id),
            settings: data
                .settings
                .iter()
                .map(|setting| SavedSetting {
                    name: setting.name().to_string(),
                    value: SettingValue::capture(setting),
                })
                .collect(),
        });
    }

    config.panels = PANEL_POS
        .iter()
        .map(|entry| PanelConfig {
            category: *entry.key(),
            x: entry.value()[0],
            y: entry.value()[1],
        })
        .collect();
    config.hud_elements = HUD_POS
        .iter()
        .map(|entry| HudConfig {
            id: entry.key().clone(),
            x: entry.value()[0],
            y: entry.value()[1],
        })
        .collect();

    config
}

fn apply_config(config: Config) {
    crate::graphic::theme::set_preset(config.theme_preset);
    crate::graphic::theme::set_custom_themes(config.custom_themes);
    if let Some(name) = config.active_custom_theme.as_deref() {
        if !crate::graphic::theme::apply_custom_theme(name) {
            log::warn!("config: saved custom theme '{name}' was not found");
        }
    }
    set_gui_open_key(config.gui_open_key);
    FAVORITES.clear();
    for favorite in config.favorites {
        FAVORITES.insert(favorite, true);
    }

    for handle in client().modules.handles() {
        let Ok(mut module) = handle.lock() else {
            continue;
        };
        let id = module.get_module_data().id;
        let Some(saved) = config.modules.iter().find(|entry| entry.id == id) else {
            continue;
        };

        let was_enabled = module.get_module_data().enabled;
        {
            let data = module.get_module_data_mut();
            data.key_bind = KeyboardKey::from(saved.key_bind);
            for setting in &mut data.settings {
                if let Some(saved_setting) = saved
                    .settings
                    .iter()
                    .find(|entry| entry.name == setting.name())
                {
                    saved_setting.value.apply(setting);
                }
            }
            data.set_enabled(saved.enabled);
        }

        if client().modules.is_active() {
            let transition = match (was_enabled, saved.enabled) {
                (false, true) => Some(module.on_start()),
                (true, false) => Some(module.on_stop()),
                _ => None,
            };
            if let Some(Err(error)) = transition {
                log::error!(
                    "config: module '{}' failed to transition while loading profile: {error}",
                    module.get_module_data().name()
                );
            }
        }
        set_expanded(id, saved.expanded);
    }

    for panel in &config.panels {
        set_panel_pos(panel.category, [panel.x, panel.y]);
    }
    HUD_POS.clear();
    for element in &config.hud_elements {
        set_hud_pos(&element.id, [element.x, element.y]);
    }
}

pub fn save() {
    let config = capture_config();
    let path = config_path();
    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            if let Err(error) = std::fs::write(&path, json) {
                log::warn!("config: could not write {}: {error}", path.display());
            }
        }
        Err(error) => log::warn!("config: could not serialize: {error}"),
    }
}

pub fn save_profile(name: &str) -> Result<()> {
    let path = profile_path(name)?;
    let directory = base_dir().join(PROFILES_DIR);
    std::fs::create_dir_all(&directory)
        .with_context(|| format!("config: could not create profile directory for '{name}'"))?;
    let json = serde_json::to_string_pretty(&capture_config())
        .context("config: could not serialize profile")?;
    std::fs::write(&path, json).with_context(|| {
        format!(
            "config: could not write profile '{}' ({})",
            name,
            path.display()
        )
    })
}

pub fn load_profile(name: &str) -> Result<()> {
    let path = profile_path(name)?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "config: could not read profile '{}' ({})",
            name,
            path.display()
        )
    })?;
    let config: Config = serde_json::from_str(&json).with_context(|| {
        format!(
            "config: could not parse profile '{}' ({})",
            name,
            path.display()
        )
    })?;
    apply_config(config);
    Ok(())
}

pub fn list_profiles() -> Result<Vec<String>> {
    let directory = base_dir().join(PROFILES_DIR);
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error).with_context(|| {
                format!("config: could not list profiles in {}", directory.display())
            })
        }
    };

    let mut profiles = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| {
            format!(
                "config: could not read profile entry in {}",
                directory.display()
            )
        })?;
        if !entry
            .file_type()
            .with_context(|| format!("config: could not inspect {}", entry.path().display()))?
            .is_file()
        {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if validate_profile_name(name).is_ok() {
            profiles.push(name.to_string());
        }
    }
    profiles.sort();
    Ok(profiles)
}

pub fn delete_profile(name: &str) -> Result<()> {
    let path = profile_path(name)?;
    std::fs::remove_file(&path).with_context(|| {
        format!(
            "config: could not delete profile '{}' ({})",
            name,
            path.display()
        )
    })
}

pub fn load() {
    let path = config_path();
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            log::warn!("config: could not read {}: {error}", path.display());
            return;
        }
    };
    let config: Config = match serde_json::from_str(&json) {
        Ok(config) => config,
        Err(error) => {
            log::warn!("config: could not parse {}: {error}", path.display());
            return;
        }
    };

    #[cfg(test)]
    mod tests {
        use super::{gui_open_key, set_gui_open_key, DEFAULT_GUI_OPEN_KEY};

        #[test]
        fn empty_gui_shortcut_falls_back_to_right_shift() {
            set_gui_open_key(-1);
            assert_eq!(gui_open_key(), DEFAULT_GUI_OPEN_KEY);
            set_gui_open_key(65);
            assert_eq!(gui_open_key(), 65);
            set_gui_open_key(DEFAULT_GUI_OPEN_KEY);
        }
    }
    apply_config(config);
}

#[cfg(test)]
mod profile_tests {
    use super::validate_profile_name;

    #[test]
    fn profile_names_reject_paths_and_empty_values() {
        assert!(validate_profile_name("pvp-1").is_ok());
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("../outside").is_err());
        assert!(validate_profile_name("name with spaces").is_err());
        assert!(validate_profile_name(&"a".repeat(65)).is_err());
    }
}
