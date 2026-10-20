use crate::config;
use crate::graphic::theme;
use crate::log_buffer;
use crate::module::{KeyboardKey, ModuleId};
use crate::state::client;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static CWD: Mutex<Option<PathBuf>> = Mutex::new(None);
static LOG_FILTER: Mutex<String> = Mutex::new(String::new());
static AUR_REQUEST_ACTIVE: AtomicBool = AtomicBool::new(false);
static TERMINAL_VISUAL: Mutex<Option<TerminalVisual>> = Mutex::new(None);

const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_COMMAND_BYTES: usize = 1024 * 1024;
const MAX_AUR_RESPONSE_BYTES: u64 = 1024 * 1024;
const MAX_OUTPUT_ENTRIES: usize = 200;
const MAX_TREE_DEPTH: usize = 3;
const MAX_AUR_RESULTS: usize = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalVisual {
    Matrix,
    Bonsai,
}

pub fn terminal_visual() -> Option<TerminalVisual> {
    TERMINAL_VISUAL.lock().ok().and_then(|visual| *visual)
}

fn set_terminal_visual(visual: Option<TerminalVisual>) {
    match TERMINAL_VISUAL.lock() {
        Ok(mut current) => *current = visual,
        Err(_) => log_buffer::push("ERROR", "terminal visual state lock poisoned"),
    }
}
const AUR_CATALOG: &[&str] = &[
    "abtop",
    "amdtop",
    "angryoxide",
    "aria2tui",
    "atuin-git",
    "bagels",
    "basalt",
    "benben",
    "cbonsai",
    "cbonsai-git",
    "cava-git",
    "cmatrix-git",
    "cmatrix-neo-git",
    "gotop",
    "ncmatrix",
];

fn sandbox_root() -> Result<PathBuf, String> {
    let root = config::base_dir().join("helium_sandbox");
    fs::create_dir_all(&root).map_err(|error| format!("sandbox: {error}"))?;
    root.canonicalize()
        .map_err(|error| format!("sandbox: {error}"))
}

fn cwd() -> Result<PathBuf, String> {
    let mut current = CWD
        .lock()
        .map_err(|_| "shell: working-directory lock poisoned".to_string())?;
    if let Some(path) = current.as_ref() {
        return Ok(path.clone());
    }
    let root = sandbox_root()?;
    *current = Some(root.clone());
    Ok(root)
}

fn set_cwd(path: PathBuf) -> Result<(), String> {
    *CWD.lock()
        .map_err(|_| "shell: working-directory lock poisoned".to_string())? = Some(path);
    Ok(())
}

fn relative_to_root(path: &Path) -> String {
    let Ok(root) = sandbox_root() else {
        return "/".to_string();
    };
    path.strip_prefix(root)
        .map(|path| {
            let path = path.to_string_lossy();
            if path.is_empty() {
                "/".to_string()
            } else {
                format!("/{path}")
            }
        })
        .unwrap_or_else(|_| "/".to_string())
}

fn resolve_in_sandbox(input: &str) -> Result<PathBuf, String> {
    let root = sandbox_root()?;
    let current = cwd()?;
    let mut relative = if input.starts_with('/') {
        PathBuf::new()
    } else {
        current
            .strip_prefix(&root)
            .map_err(|_| "permission denied: current directory escaped sandbox".to_string())?
            .to_path_buf()
    };
    let input = input.trim_start_matches('/');

    for component in Path::new(input).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(name) => relative.push(name),
            Component::ParentDir => {
                if !relative.pop() {
                    return Err("permission denied: path is outside sandbox".to_string());
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("permission denied: invalid sandbox path".to_string());
            }
        }
    }

    let resolved = root.join(relative);
    let mut current = root;
    let remainder = resolved
        .strip_prefix(&current)
        .map_err(|_| "permission denied: path is outside sandbox".to_string())?
        .to_path_buf();
    for component in remainder.components() {
        if let Component::Normal(name) = component {
            current.push(name);
            match fs::symlink_metadata(&current) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err("permission denied: symbolic links are not allowed".to_string());
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => return Err(format!("path: {error}")),
            }
        }
    }
    Ok(resolved)
}

pub fn prompt() -> String {
    match cwd() {
        Ok(path) => format!("helium:{} $", relative_to_root(&path)),
        Err(_) => "helium:/ $".to_string(),
    }
}

pub fn log_filter() -> String {
    LOG_FILTER
        .lock()
        .map(|filter| filter.clone())
        .unwrap_or_default()
}

fn set_log_filter(filter: String) -> Result<(), String> {
    *LOG_FILTER
        .lock()
        .map_err(|_| "shell: log-filter lock poisoned".to_string())? = filter;
    Ok(())
}

pub fn clear_log_filter() {
    if let Err(error) = set_log_filter(String::new()) {
        log_buffer::push("ERROR", error);
    }
}

pub fn execute(line: &str) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    if line.len() > MAX_COMMAND_BYTES || line.chars().any(char::is_control) {
        log_buffer::push(
            "ERROR",
            "command rejected: input is too long or contains control characters",
        );
        return;
    }

    let (command, args) = split_command(line);
    match command.to_ascii_lowercase().as_str() {
        "help" | "?" => help(),
        "clear" | "cls" => {
            log_buffer::clear();
            clear_log_filter();
        }
        "list" | "modules" => list_modules(),
        "toggle" => {
            if let Some(name) = args {
                set_module(name, None);
            } else {
                usage("toggle <module>");
            }
        }
        "enable" => {
            if let Some(name) = args {
                set_module(name, Some(true));
            } else {
                usage("enable <module>");
            }
        }
        "disable" => {
            if let Some(name) = args {
                set_module(name, Some(false));
            } else {
                usage("disable <module>");
            }
        }
        "bind" => command_bind(args),
        "search" => command_search(args),
        "echo" => log_buffer::push("INFO", args.unwrap_or_default()),
        "presets" => list_presets(),
        "preset" => command_preset(args),
        "aur" => command_aur(args),
        "cmatrix" => {
            set_terminal_visual(Some(TerminalVisual::Matrix));
            log_buffer::push(
                "INFO",
                "cmatrix visual started; type stop to return to logs",
            );
        }
        "cbonsai" => {
            set_terminal_visual(Some(TerminalVisual::Bonsai));
            log_buffer::push(
                "INFO",
                "cbonsai visual started; type stop to return to logs",
            );
        }
        "stop" | "exit" => {
            if terminal_visual().is_some() {
                set_terminal_visual(None);
                log_buffer::push("INFO", "terminal visual stopped");
            } else {
                log_buffer::push("INFO", "no terminal visual is running");
            }
        }
        "panic" => {
            log_buffer::push("WARN", "Panic unload requested");
            std::thread::spawn(crate::graphic::ui_engine::call_panic);
        }
        "pwd" => match cwd() {
            Ok(path) => log_buffer::push("INFO", relative_to_root(&path)),
            Err(error) => log_buffer::push("ERROR", error),
        },
        "cd" => command_cd(args),
        "ls" | "dir" => command_ls(args),
        "cat" | "type" => command_cat(args),
        "mkdir" => command_mkdir(args),
        "touch" => command_touch(args),
        "write" => command_write(args),
        "tree" => command_tree(args),
        "du" => command_du(args),
        "head" => command_head(args),
        "wc" => command_wc(args),
        "whoami" => log_buffer::push("INFO", "helium (sandbox)"),
        "uname" => log_buffer::push("INFO", "HeliumShell sandbox"),
        "date" => log_buffer::push("INFO", format!("{:?}", std::time::SystemTime::now())),
        "sudo" | "doas" | "su" | "rm" | "rmdir" | "unlink" | "shred" | "dd" | "mkfs" | "fdisk"
        | "parted" | "wipefs" | "chmod" | "chown" | "mv" | "kill" | "killall" | "pkill"
        | "reboot" | "shutdown" | "poweroff" | "bash" | "sh" | "zsh" | "fish" | "python"
        | "python3" | "perl" | "ruby" | "node" | "curl" | "wget" | "ssh" | "scp" | "rsync" => {
            log_buffer::push(
                "ERROR",
                "blocked: command is unavailable; terminal is sandbox-only",
            );
        }
        _ => log_buffer::push(
            "ERROR",
            format!("unknown command: {command}; type help for commands"),
        ),
    }
}

fn split_command(line: &str) -> (&str, Option<&str>) {
    match line.find(char::is_whitespace) {
        Some(index) => (&line[..index], Some(line[index..].trim())),
        None => (line, None),
    }
}

fn usage(text: &str) {
    log_buffer::push("WARN", format!("usage: {text}"));
}

fn help() {
    for line in [
        "Helium Terminal commands:",
        "  help                         Show this help",
        "  clear                        Clear log buffer and search filter",
        "  list / modules               List modules and ON/OFF state",
        "  toggle <module>              Toggle a module",
        "  enable <module>              Enable a module",
        "  disable <module>             Disable a module",
        "  bind <module> [key]          Show or set a module keybind",
        "  search <text>                Filter log messages (search clear removes filter)",
        "  echo <text>                  Print a message",
        "  presets                      List theme presets",
        "  preset <name>                Apply a theme preset",
        "  cmatrix                      Show the animated Matrix rain visual",
        "  cbonsai                      Show an animated bonsai tree",
        "  stop / exit                  Stop the current terminal visual",
        "  aur search <query>           Search AUR package names and descriptions",
        "  aur info <package>           Show AUR metadata and package URL",
        "  aur list [filter]            List the curated AUR terminal-app catalog",
        "  panic                        Unload Helium immediately",
        "  pwd, cd, ls, cat, mkdir      Sandbox filesystem commands (max file: 1 MiB)",
        "  touch, write, tree, head, wc, du",
        "Filesystem commands only access helium_sandbox/; no OS shell is executed.",
    ] {
        log_buffer::push("INFO", line);
    }
}

fn command_aur(args: Option<&str>) {
    let Some(args) = args else {
        usage("aur search <query> | aur info <package> | aur list [filter]");
        return;
    };
    let (action, query) = split_command(args);
    let mode = match action.to_ascii_lowercase().as_str() {
        "search" => "search",
        "info" => "info",
        "list" => "list",
        _ => {
            usage("aur search <query> | aur info <package> | aur list [filter]");
            return;
        }
    };
    let query = query.filter(|query| !query.is_empty());
    if mode != "list" && query.is_none() {
        usage(if mode == "search" {
            "aur search <query>"
        } else {
            "aur info <package>"
        });
        return;
    }
    let query = query.unwrap_or_default();
    if query.len() > 256 || query.chars().any(char::is_control) {
        log_buffer::push(
            "ERROR",
            "AUR query must be at most 256 bytes and contain no control characters",
        );
        return;
    }
    if AUR_REQUEST_ACTIVE.swap(true, Ordering::AcqRel) {
        log_buffer::push("WARN", "an AUR request is already running");
        return;
    }

    let mode = mode.to_string();
    let query = query.to_string();
    if mode == "list" {
        if query.is_empty() {
            log_buffer::push("INFO", "AUR terminal-app catalog request started");
        } else {
            log_buffer::push("INFO", format!("AUR terminal-app catalog filter: {query}"));
        }
    } else {
        log_buffer::push("INFO", format!("AUR {mode} request started: {query}"));
    }
    std::thread::spawn(move || {
        struct RequestGuard;
        impl Drop for RequestGuard {
            fn drop(&mut self) {
                AUR_REQUEST_ACTIVE.store(false, Ordering::Release);
            }
        }
        let _guard = RequestGuard;
        let result = aur_request(&mode, &query);
        match result {
            Ok(entries) if entries.is_empty() => {
                if mode == "list" {
                    log_buffer::push("INFO", "AUR: no catalog packages match that filter");
                } else {
                    log_buffer::push("INFO", format!("AUR: no package results for '{query}'"));
                }
            }
            Ok(entries) => {
                if mode == "list" {
                    log_buffer::push(
                        "INFO",
                        format!("AUR terminal apps ({} found):", entries.len()),
                    );
                    log_buffer::push(
                        "INFO",
                        "cmatrix is in Arch repos; the AUR variant is cmatrix-git.",
                    );
                }
                for entry in entries.into_iter().take(MAX_AUR_RESULTS) {
                    log_buffer::push("INFO", entry);
                }
            }
            Err(error) => log_buffer::push("ERROR", format!("AUR: {error}")),
        }
    });
}

fn aur_request(mode: &str, query: &str) -> Result<Vec<String>, String> {
    let url = match mode {
        "search" => format!(
            "https://aur.archlinux.org/rpc/v5/search/{}?by=name-desc",
            percent_encode(query)
        ),
        "info" => format!(
            "https://aur.archlinux.org/rpc/v5/info?arg%5B%5D={}",
            percent_encode(query)
        ),
        "list" => {
            let mut url = String::from("https://aur.archlinux.org/rpc/v5/info?");
            for (index, name) in AUR_CATALOG.iter().enumerate() {
                if index > 0 {
                    url.push('&');
                }
                url.push_str("arg%5B%5D=");
                url.push_str(name);
            }
            url
        }
        _ => return Err("unsupported operation".to_string()),
    };
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(8))
        .timeout_write(std::time::Duration::from_secs(5))
        .build();
    let response = agent
        .get(&url)
        .set("User-Agent", "HeliumClient/0.1 AUR read-only lookup")
        .call()
        .map_err(|error| format!("request failed: {error}"))?;
    let mut body = Vec::new();
    response
        .into_reader()
        .take(MAX_AUR_RESPONSE_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|error| format!("response read failed: {error}"))?;
    if body.len() as u64 > MAX_AUR_RESPONSE_BYTES {
        return Err("response exceeded the 1 MiB limit".to_string());
    }
    let response: serde_json::Value =
        serde_json::from_slice(&body).map_err(|error| format!("invalid API response: {error}"))?;
    let response_type = response.get("type").and_then(serde_json::Value::as_str);
    if !matches!(response_type, Some("search" | "info" | "multiinfo")) {
        return Err(response
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unexpected API response")
            .to_string());
    }
    let results = response
        .get("results")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "API response has no results list".to_string())?;
    Ok(results
        .iter()
        .filter(|package| {
            mode != "list"
                || query.is_empty()
                || package
                    .get("Name")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|name| {
                        name.to_ascii_lowercase()
                            .contains(&query.to_ascii_lowercase())
                    })
                || package
                    .get("Description")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|description| {
                        description
                            .to_ascii_lowercase()
                            .contains(&query.to_ascii_lowercase())
                    })
        })
        .take(MAX_AUR_RESULTS)
        .map(format_aur_package)
        .collect())
}

fn format_aur_package(package: &serde_json::Value) -> String {
    let string = |key| {
        package
            .get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("?")
    };
    let number = |key| {
        package
            .get(key)
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default()
    };
    let name = string("Name");
    let package_url = package
        .get("URL")
        .and_then(serde_json::Value::as_str)
        .filter(|url| !url.is_empty())
        .unwrap_or("no upstream URL");
    let popularity = package
        .get("Popularity")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_default();
    format!(
        "{name} {} | votes: {} | popularity: {popularity:.2} | maintainer: {} | upstream: {package_url} | AUR: https://aur.archlinux.org/packages/{name} | {}",
        string("Version"),
        number("NumVotes"),
        string("Maintainer"),
        string("Description")
    )
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn module_id(name: &str) -> Option<ModuleId> {
    let target = normalize_name(name);
    let alias = match target.as_str() {
        "aura" | "killaura" => Some(ModuleId::KillAura),
        "mob" | "mobaura" => Some(ModuleId::MobAura),
        "playeresp" | "esp" => Some(ModuleId::PlayerEsp),
        "chestesp" => Some(ModuleId::ChestEsp),
        "holeesp" => Some(ModuleId::HoleEsp),
        "mineral" | "mineralesp" | "xray" => Some(ModuleId::MineralEsp),
        "soundfx" => Some(ModuleId::SoundFX),
        "antiafk" => Some(ModuleId::AntiAfk),
        "autototem" => Some(ModuleId::AutoTotem),
        "autoeat" => Some(ModuleId::AutoEat),
        "surround" => Some(ModuleId::Surround),
        "autoarmor" => Some(ModuleId::AutoArmor),
        "cheststealer" => Some(ModuleId::ChestStealer),
        "triggerbot" => Some(ModuleId::TriggerBot),
        "autocrystal" => Some(ModuleId::AutoCrystal),
        "nofall" => Some(ModuleId::NoFall),
        "noslowdown" => Some(ModuleId::NoSlowdown),
        "nojumpdelay" => Some(ModuleId::NoJumpDelay),
        "safewalk" => Some(ModuleId::SafeWalk),
        "antivoid" => Some(ModuleId::AntiVoid),
        "blink" => Some(ModuleId::Blink),
        "hud" | "hudeditor" => Some(ModuleId::HudEditor),
        "norender" => Some(ModuleId::NoRender),
        "notifications" => Some(ModuleId::Notifications),
        "terminal" => Some(ModuleId::Terminal),
        _ => None,
    };
    alias.or_else(|| {
        client()
            .modules
            .by_id()
            .keys()
            .find(|id| normalize_name(id.display_name()) == target)
            .copied()
    })
}

fn set_module(name: &str, requested: Option<bool>) {
    let Some(id) = module_id(name) else {
        log_buffer::push("ERROR", format!("module not found: {name}"));
        return;
    };
    let Some(handle) = client().modules.get(id) else {
        log_buffer::push(
            "ERROR",
            format!("module not registered: {}", id.display_name()),
        );
        return;
    };
    let Ok(mut module) = handle.lock() else {
        log_buffer::push(
            "ERROR",
            format!("module lock poisoned: {}", id.display_name()),
        );
        return;
    };
    let was_enabled = module.get_module_data().enabled;
    let enabled = requested.unwrap_or(!was_enabled);
    if enabled == was_enabled {
        log_buffer::push(
            "INFO",
            format!(
                "{} is already {}",
                id.display_name(),
                if enabled { "ON" } else { "OFF" }
            ),
        );
        return;
    }
    if client().modules.is_active() {
        let result = if enabled {
            module.on_start()
        } else {
            module.on_stop()
        };
        if let Err(error) = result {
            log_buffer::push(
                "ERROR",
                format!(
                    "{} failed to {}: {error}",
                    id.display_name(),
                    if enabled { "start" } else { "stop" }
                ),
            );
            return;
        }
    }
    module.get_module_data_mut().set_enabled(enabled);
    drop(module);
    crate::config::save();
    log_buffer::push(
        "INFO",
        format!(
            "{} {}",
            id.display_name(),
            if enabled { "ON" } else { "OFF" }
        ),
    );
}

fn list_modules() {
    let mut modules: Vec<_> = client().modules.by_id().into_iter().collect();
    modules.sort_by_key(|(id, _)| id.display_name());
    for (id, handle) in modules {
        let Ok(module) = handle.lock() else {
            log_buffer::push(
                "ERROR",
                format!("module lock poisoned: {}", id.display_name()),
            );
            continue;
        };
        log_buffer::push(
            "INFO",
            format!(
                "{} [{}]",
                id.display_name(),
                if module.get_module_data().enabled {
                    "ON"
                } else {
                    "OFF"
                }
            ),
        );
    }
}

fn command_bind(args: Option<&str>) {
    let Some(args) = args else {
        usage("bind <module> [key]");
        return;
    };
    let mut parts = args.split_whitespace();
    let Some(name) = parts.next() else {
        usage("bind <module> [key]");
        return;
    };
    let Some(id) = module_id(name) else {
        log_buffer::push("ERROR", format!("module not found: {name}"));
        return;
    };
    let Some(handle) = client().modules.get(id) else {
        log_buffer::push(
            "ERROR",
            format!("module not registered: {}", id.display_name()),
        );
        return;
    };
    let Ok(mut module) = handle.lock() else {
        log_buffer::push(
            "ERROR",
            format!("module lock poisoned: {}", id.display_name()),
        );
        return;
    };

    if let Some(key) = parts.next() {
        if parts.next().is_some() {
            usage("bind <module> [key]");
            return;
        }
        let Some(keycode) = keycode(key) else {
            log_buffer::push("ERROR", format!("unsupported key: {key}"));
            return;
        };
        module.get_module_data_mut().key_bind = KeyboardKey::from(keycode);
        let label = module.get_module_data().key_bind.to_string();
        drop(module);
        crate::config::save();
        log_buffer::push("INFO", format!("{} bound to {label}", id.display_name()));
    } else {
        log_buffer::push(
            "INFO",
            format!(
                "{} keybind: {}",
                id.display_name(),
                module.get_module_data().key_bind
            ),
        );
    }
}

fn keycode(key: &str) -> Option<i32> {
    let key = key.to_ascii_uppercase();
    if key.len() == 1 {
        let byte = key.as_bytes()[0];
        if byte.is_ascii_alphabetic() || byte.is_ascii_digit() {
            return Some(i32::from(byte));
        }
    }
    match key.as_str() {
        "NONE" => Some(-1),
        "ESC" | "ESCAPE" => Some(256),
        "ENTER" | "RETURN" => Some(257),
        "TAB" => Some(258),
        "BACKSPACE" | "BACK" => Some(259),
        "INSERT" => Some(260),
        "DELETE" => Some(261),
        "RIGHT" => Some(262),
        "LEFT" => Some(263),
        "DOWN" => Some(264),
        "UP" => Some(265),
        "PAGEUP" => Some(266),
        "PAGEDOWN" => Some(267),
        "HOME" => Some(268),
        "END" => Some(269),
        "CAPSLOCK" | "CAPITAL" => Some(280),
        "F1" => Some(290),
        "F2" => Some(291),
        "F3" => Some(292),
        "F4" => Some(293),
        "F5" => Some(294),
        "F6" => Some(295),
        "F7" => Some(296),
        "F8" => Some(297),
        "F9" => Some(298),
        "F10" => Some(299),
        "F11" => Some(300),
        "F12" => Some(301),
        "LSHIFT" => Some(340),
        "LCTRL" | "LCONTROL" => Some(341),
        "LALT" => Some(342),
        "RSHIFT" | "SHIFT" => Some(344),
        "RCTRL" | "RCONTROL" => Some(345),
        "SPACE" => Some(32),
        _ => None,
    }
}

fn command_search(args: Option<&str>) {
    let filter = args.unwrap_or_default();
    let filter = if filter.eq_ignore_ascii_case("clear") {
        String::new()
    } else {
        filter.to_string()
    };
    match set_log_filter(filter.clone()) {
        Ok(()) if filter.is_empty() => log_buffer::push("INFO", "log filter cleared"),
        Ok(()) => log_buffer::push("INFO", format!("filtering logs for: {filter}")),
        Err(error) => log_buffer::push("ERROR", error),
    }
}

fn list_presets() {
    for (index, preset) in theme::PRESETS.iter().enumerate() {
        let current = if theme::selected_preset() == index {
            " (selected)"
        } else {
            ""
        };
        log_buffer::push("INFO", format!("{}{}", preset.name, current));
    }
}

fn command_preset(args: Option<&str>) {
    let Some(name) = args.map(str::trim).filter(|name| !name.is_empty()) else {
        usage("preset <name>");
        return;
    };
    let target = normalize_name(name);
    let Some((index, preset)) = theme::PRESETS
        .iter()
        .enumerate()
        .find(|(_, preset)| normalize_name(preset.name) == target)
    else {
        log_buffer::push(
            "ERROR",
            format!("theme preset not found: {name}; use presets"),
        );
        return;
    };
    theme::set_preset(index);
    crate::config::save();
    log_buffer::push("INFO", format!("theme preset applied: {}", preset.name));
}

fn command_cd(args: Option<&str>) {
    let target = args.unwrap_or("/");
    let path = match resolve_in_sandbox(target) {
        Ok(path) => path,
        Err(error) => {
            log_buffer::push("ERROR", error);
            return;
        }
    };
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_dir() => match set_cwd(path.clone()) {
            Ok(()) => log_buffer::push("INFO", relative_to_root(&path)),
            Err(error) => log_buffer::push("ERROR", error),
        },
        Ok(_) => log_buffer::push("WARN", format!("not a directory: {target}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            log_buffer::push("WARN", format!("no such directory: {target}"));
        }
        Err(error) => log_buffer::push("ERROR", format!("cd: {error}")),
    }
}

fn command_ls(args: Option<&str>) {
    let target = args.unwrap_or(".");
    match resolve_in_sandbox(target) {
        Ok(path) if path.is_file() => log_buffer::push("INFO", relative_to_root(&path)),
        Ok(path) => match fs::read_dir(&path) {
            Ok(entries) => {
                let mut entries: Vec<_> = match entries
                    .take(MAX_OUTPUT_ENTRIES + 1)
                    .collect::<std::io::Result<Vec<_>>>()
                {
                    Ok(entries) => entries,
                    Err(error) => {
                        log_buffer::push("ERROR", format!("ls: {error}"));
                        return;
                    }
                };
                entries.sort_by_key(|entry| entry.file_name());
                if entries.is_empty() {
                    log_buffer::push("INFO", "(empty)");
                }
                for entry in entries.iter().take(MAX_OUTPUT_ENTRIES) {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let suffix = if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                        "/"
                    } else {
                        ""
                    };
                    log_buffer::push("INFO", format!("{name}{suffix}"));
                }
                if entries.len() > MAX_OUTPUT_ENTRIES {
                    log_buffer::push("WARN", "listing truncated");
                }
            }
            Err(error) => log_buffer::push("ERROR", format!("ls: {error}")),
        },
        Err(error) => log_buffer::push("ERROR", format!("ls: {error}")),
    }
}

fn read_file(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("file: {error}"))?;
    if !metadata.is_file() {
        return Err("not a regular file".to_string());
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err("file too large (max 1 MiB)".to_string());
    }
    let file = fs::File::open(path).map_err(|error| format!("file: {error}"))?;
    let mut content = String::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|error| format!("file: {error}"))?;
    if content.len() as u64 > MAX_FILE_BYTES {
        return Err("file too large (max 1 MiB)".to_string());
    }
    Ok(content)
}

fn command_cat(args: Option<&str>) {
    let Some(path) = args else {
        usage("cat <file>");
        return;
    };
    match resolve_in_sandbox(path).and_then(|path| read_file(&path)) {
        Ok(content) => {
            for line in content.lines().take(MAX_OUTPUT_ENTRIES) {
                log_buffer::push("INFO", line);
            }
        }
        Err(error) => log_buffer::push("ERROR", format!("cat: {error}")),
    }
}

fn command_mkdir(args: Option<&str>) {
    let Some(path) = args else {
        usage("mkdir <directory>");
        return;
    };
    match resolve_in_sandbox(path).and_then(|path| {
        fs::create_dir_all(&path).map_err(|error| format!("mkdir: {error}"))?;
        Ok(path)
    }) {
        Ok(path) => log_buffer::push("INFO", format!("created {}", relative_to_root(&path))),
        Err(error) => log_buffer::push("ERROR", error),
    }
}

fn command_touch(args: Option<&str>) {
    let Some(path) = args else {
        usage("touch <file>");
        return;
    };
    let result = resolve_in_sandbox(path).and_then(|path| {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("touch: {error}"))?;
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| format!("touch: {error}"))?;
        Ok(path)
    });
    match result {
        Ok(path) => log_buffer::push("INFO", format!("touched {}", relative_to_root(&path))),
        Err(error) => log_buffer::push("ERROR", error),
    }
}

fn command_write(args: Option<&str>) {
    let Some(args) = args else {
        usage("write <file> <text>");
        return;
    };
    let Some((path, text)) = args.split_once(char::is_whitespace) else {
        usage("write <file> <text>");
        return;
    };
    let text = text.trim_start();
    if text.len() as u64 >= MAX_FILE_BYTES {
        log_buffer::push("ERROR", "write: text exceeds 1 MiB limit");
        return;
    }
    match resolve_in_sandbox(path).and_then(|path| {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("write: {error}"))?;
        }
        fs::write(&path, format!("{text}\n")).map_err(|error| format!("write: {error}"))?;
        Ok(path)
    }) {
        Ok(path) => log_buffer::push("INFO", format!("wrote {}", relative_to_root(&path))),
        Err(error) => log_buffer::push("ERROR", error),
    }
}

fn command_tree(args: Option<&str>) {
    let target = args.unwrap_or(".");
    match resolve_in_sandbox(target) {
        Ok(path) if path.is_dir() => {
            let mut remaining = MAX_OUTPUT_ENTRIES;
            walk_tree(&path, 0, &mut remaining);
        }
        Ok(path) => log_buffer::push("INFO", relative_to_root(&path)),
        Err(error) => log_buffer::push("ERROR", format!("tree: {error}")),
    }
}

fn walk_tree(path: &Path, depth: usize, remaining: &mut usize) {
    if depth > MAX_TREE_DEPTH || *remaining == 0 {
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            log_buffer::push("ERROR", format!("tree: {error}"));
            return;
        }
    };
    let available = *remaining;
    let mut entries: Vec<_> = match entries
        .take(available.saturating_add(1))
        .collect::<std::io::Result<Vec<_>>>()
    {
        Ok(entries) => entries,
        Err(error) => {
            log_buffer::push("ERROR", format!("tree: {error}"));
            return;
        }
    };
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries.iter().take(*remaining) {
        *remaining -= 1;
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(error) => {
                log_buffer::push("ERROR", format!("tree: {error}"));
                continue;
            }
        };
        log_buffer::push(
            "INFO",
            format!(
                "{}{name}{}",
                "  ".repeat(depth),
                if kind.is_dir() { "/" } else { "" }
            ),
        );
        if kind.is_dir() {
            walk_tree(&entry.path(), depth + 1, remaining);
        }
    }
    if entries.len() > available || *remaining == 0 {
        log_buffer::push("WARN", "tree listing truncated");
    }
}

fn command_du(args: Option<&str>) {
    let target = args.unwrap_or(".");
    match resolve_in_sandbox(target).and_then(|path| {
        let size = directory_size(&path)?;
        Ok((path, size))
    }) {
        Ok((path, size)) => {
            log_buffer::push("INFO", format!("{}  {size} bytes", relative_to_root(&path)))
        }
        Err(error) => log_buffer::push("ERROR", format!("du: {error}")),
    }
}

fn directory_size(path: &Path) -> Result<u64, String> {
    let mut remaining = MAX_OUTPUT_ENTRIES;
    directory_size_at_depth(path, 0, &mut remaining)
}

fn directory_size_at_depth(
    path: &Path,
    depth: usize,
    remaining: &mut usize,
) -> Result<u64, String> {
    if depth > MAX_TREE_DEPTH || *remaining == 0 {
        return Ok(0);
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("du: {error}"))?;
    if metadata.file_type().is_symlink() {
        return Err("du: symbolic links are not allowed".to_string());
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let entries = fs::read_dir(path).map_err(|error| format!("du: {error}"))?;
    let mut size = 0u64;
    for entry in entries.take(*remaining) {
        *remaining -= 1;
        let entry = entry.map_err(|error| format!("du: {error}"))?;
        let kind = entry.file_type().map_err(|error| format!("du: {error}"))?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_file() {
            size = size.saturating_add(
                entry
                    .metadata()
                    .map_err(|error| format!("du: {error}"))?
                    .len(),
            );
        } else if kind.is_dir() {
            size = size.saturating_add(directory_size_at_depth(
                &entry.path(),
                depth + 1,
                remaining,
            )?);
        }
    }
    Ok(size)
}

fn command_head(args: Option<&str>) {
    let Some(path) = args else {
        usage("head <file>");
        return;
    };
    match resolve_in_sandbox(path).and_then(|path| read_file(&path)) {
        Ok(content) => {
            for line in content.lines().take(10) {
                log_buffer::push("INFO", line);
            }
        }
        Err(error) => log_buffer::push("ERROR", format!("head: {error}")),
    }
}

fn command_wc(args: Option<&str>) {
    let Some(name) = args else {
        usage("wc <file>");
        return;
    };
    match resolve_in_sandbox(name).and_then(|path| read_file(&path).map(|content| (path, content)))
    {
        Ok((path, content)) => log_buffer::push(
            "INFO",
            format!(
                "{} {} {} {}",
                content.lines().count(),
                content.split_whitespace().count(),
                content.len(),
                relative_to_root(&path)
            ),
        ),
        Err(error) => log_buffer::push("ERROR", format!("wc: {error}")),
    }
}
