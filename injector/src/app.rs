use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::inject;
use crate::platform::{find_minecraft_processes, InjectError, ProcessInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppView {
    Main,
    Settings,
    News,
}

#[derive(Debug, Clone)]
pub enum InjectionStatus {
    Idle,
    Scanning,
    Injecting(u32),
    Done(u32),
    Failed(String),
}

impl InjectionStatus {
    pub fn message(&self) -> String {
        match self {
            InjectionStatus::Idle => "Ready.".to_string(),
            InjectionStatus::Scanning => "Scanning for Minecraft…".to_string(),
            InjectionStatus::Injecting(pid) => format!("Injecting into process {pid}…"),
            InjectionStatus::Done(pid) => format!("Injected into process {pid}."),
            InjectionStatus::Failed(reason) => format!("Error: {reason}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_scan_on_start: bool,
    pub auto_select_first: bool,
    pub show_pid_in_cards: bool,
    pub panorama_speed: f32,
    pub dark_overlay: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_scan_on_start: true,
            auto_select_first: true,
            show_pid_in_cards: true,
            panorama_speed: 1.0,
            dark_overlay: 0.49,
        }
    }
}

impl AppSettings {
    fn settings_path() -> Option<PathBuf> {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                return Some(dir.join("injector_settings.json"));
            }
        }
        std::env::current_dir()
            .ok()
            .map(|d| d.join("injector_settings.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::settings_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<AppSettings>(&text) {
                Ok(mut s) => {
                    s.panorama_speed = s.panorama_speed.clamp(0.4, 2.5);
                    s.dark_overlay = s.dark_overlay.clamp(0.15, 0.75);
                    info!("settings loaded from {}", path.display());
                    s
                }
                Err(e) => {
                    warn!("could not parse settings file: {e}; using defaults");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let Some(path) = Self::settings_path() else {
            warn!("could not resolve settings path");
            return;
        };
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    warn!("failed to write settings: {e}");
                } else {
                    info!("settings saved to {}", path.display());
                }
            }
            Err(e) => warn!("failed to serialize settings: {e}"),
        }
    }
}

pub struct InjectorApp {
    processes: Vec<ProcessInfo>,
    selected_pid: Option<u32>,
    status: InjectionStatus,
    pending: Option<(u32, Receiver<Result<(), InjectError>>)>,
    view: AppView,
    settings: AppSettings,
    settings_dirty: bool,
}

impl InjectorApp {
    pub fn new() -> Self {
        let settings = AppSettings::load();
        Self {
            processes: Vec::new(),
            selected_pid: None,
            status: InjectionStatus::Idle,
            pending: None,
            view: AppView::Main,
            settings,
            settings_dirty: false,
        }
    }

    pub fn view(&self) -> &AppView {
        &self.view
    }

    pub fn set_view(&mut self, view: AppView) {
        if self.view == AppView::Settings && view != AppView::Settings {
            self.flush_settings();
        }
        self.view = view;
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        self.settings_dirty = true;
        &mut self.settings
    }

    pub fn flush_settings_if_dirty(&mut self) {
        if self.settings_dirty {
            self.flush_settings();
        }
    }

    fn flush_settings(&mut self) {
        self.settings.save();
        self.settings_dirty = false;
    }

    pub fn processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    pub fn selected_pid(&self) -> Option<u32> {
        self.selected_pid
    }

    pub fn status(&self) -> &InjectionStatus {
        &self.status
    }

    pub fn is_busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn select(&mut self, pid: u32) {
        if self.processes.iter().any(|p| p.pid == pid) {
            self.selected_pid = Some(pid);
        }
    }

    pub fn scan(&mut self) {
        if self.is_busy() {
            return;
        }
        self.status = InjectionStatus::Scanning;
        self.processes = find_minecraft_processes();

        let kept = self
            .selected_pid
            .is_some_and(|pid| self.processes.iter().any(|p| p.pid == pid));
        if !kept {
            if self.settings.auto_select_first {
                self.selected_pid = self.processes.first().map(|p| p.pid);
            } else {
                self.selected_pid = None;
            }
        }
        self.status = InjectionStatus::Idle;
    }

    pub fn start_injection(&mut self) {
        if self.is_busy() {
            return;
        }
        let Some(pid) = self.selected_pid else {
            return;
        };

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(inject::inject(pid));
        });
        self.pending = Some((pid, rx));
        self.status = InjectionStatus::Injecting(pid);
    }

    pub fn poll(&mut self) -> bool {
        let Some((pid, rx)) = &self.pending else {
            return false;
        };
        let pid = *pid;
        match rx.try_recv() {
            Ok(result) => {
                self.status = match result {
                    Ok(()) => InjectionStatus::Done(pid),
                    Err(e) => InjectionStatus::Failed(e.to_string()),
                };
                self.pending = None;
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.status =
                    InjectionStatus::Failed("injection worker stopped unexpectedly".to_string());
                self.pending = None;
                true
            }
        }
    }

    pub fn client_folder(&self) -> Option<PathBuf> {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                return Some(dir.to_path_buf());
            }
        }
        std::env::current_dir().ok()
    }
}

impl Default for InjectorApp {
    fn default() -> Self {
        Self::new()
    }
}
