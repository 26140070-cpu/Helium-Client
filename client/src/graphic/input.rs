use crate::module::ModuleId;
use crate::state::{client, minecraft};
use libloading::Library;
use log::info;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

pub static PENDING_EVENTS: Mutex<Vec<egui::Event>> = Mutex::new(Vec::new());

fn push_event(event: egui::Event) {
    if let Ok(mut queue) = PENDING_EVENTS.lock() {
        queue.push(event);
    }
}

fn translate_mods(mods: i32) -> egui::Modifiers {
    let ctrl = mods & 0x2 != 0;
    egui::Modifiers {
        alt: mods & 0x4 != 0,
        ctrl,
        shift: mods & 0x1 != 0,
        mac_cmd: false,

        command: ctrl,
    }
}

fn translate_glfw_key(key: i32) -> Option<egui::Key> {
    use egui::Key::*;
    Some(match key {
        32 => Space,
        39 => Quote,
        44 => Comma,
        45 => Minus,
        46 => Period,
        47 => Slash,
        48 => Num0,
        49 => Num1,
        50 => Num2,
        51 => Num3,
        52 => Num4,
        53 => Num5,
        54 => Num6,
        55 => Num7,
        56 => Num8,
        57 => Num9,
        59 => Semicolon,
        61 => Equals,
        65 => A,
        66 => B,
        67 => C,
        68 => D,
        69 => E,
        70 => F,
        71 => G,
        72 => H,
        73 => I,
        74 => J,
        75 => K,
        76 => L,
        77 => M,
        78 => N,
        79 => O,
        80 => P,
        81 => Q,
        82 => R,
        83 => S,
        84 => T,
        85 => U,
        86 => V,
        87 => W,
        88 => X,
        89 => Y,
        90 => Z,
        91 => OpenBracket,
        92 => Backslash,
        93 => CloseBracket,
        96 => Backtick,
        256 => Escape,
        257 => Enter,
        258 => Tab,
        259 => Backspace,
        260 => Insert,
        261 => Delete,
        262 => ArrowRight,
        263 => ArrowLeft,
        264 => ArrowDown,
        265 => ArrowUp,
        266 => PageUp,
        267 => PageDown,
        268 => Home,
        269 => End,

        320 => Num0,
        321 => Num1,
        322 => Num2,
        323 => Num3,
        324 => Num4,
        325 => Num5,
        326 => Num6,
        327 => Num7,
        328 => Num8,
        329 => Num9,
        330 => Period,
        331 => Slash,
        333 => Minus,
        334 => Plus,
        335 => Enter,
        336 => Equals,
        290..=301 => match key - 289 {
            1 => F1,
            2 => F2,
            3 => F3,
            4 => F4,
            5 => F5,
            6 => F6,
            7 => F7,
            8 => F8,
            9 => F9,
            10 => F10,
            11 => F11,
            12 => F12,
            _ => return None,
        },
        _ => return None,
    })
}

pub static GUI_OPEN: AtomicBool = AtomicBool::new(false);

pub static LAST_KEY_PRESSED: AtomicI32 = AtomicI32::new(-1);
static GUI_KEY_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub static MOUSE_STATE: Mutex<MouseState> = Mutex::new(MouseState::NEW);

#[derive(Default, Clone, Copy)]
pub struct MouseState {
    pub x: f64,
    pub y: f64,
    pub left_down: bool,
    pub right_down: bool,
    pub left_clicked: bool,
    pub right_clicked: bool,
    pub middle_clicked: bool,

    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl MouseState {
    const NEW: MouseState = MouseState {
        x: 0.0,
        y: 0.0,
        left_down: false,
        right_down: false,
        left_clicked: false,
        right_clicked: false,
        middle_clicked: false,
        scroll_x: 0.0,
        scroll_y: 0.0,
    };
}

const GLFW_RELEASE: i32 = 0;
const GLFW_PRESS: i32 = 1;
const GLFW_REPEAT: i32 = 2;
const GLFW_MOUSE_BUTTON_LEFT: i32 = 0;
const GLFW_MOUSE_BUTTON_RIGHT: i32 = 1;
const GLFW_MOUSE_BUTTON_MIDDLE: i32 = 2;
const GLFW_KEY_ESCAPE: i32 = 256;

const GLFW_CURSOR: i32 = 0x0003_3001;
const GLFW_CURSOR_NORMAL: i32 = 0x0003_4001;
const GLFW_CURSOR_DISABLED: i32 = 0x0003_4003;

type MouseButtonFun = extern "C" fn(*mut c_void, i32, i32, i32);
type CursorPosFun = extern "C" fn(*mut c_void, f64, f64);
type KeyFun = extern "C" fn(*mut c_void, i32, i32, i32, i32);
type ScrollFun = extern "C" fn(*mut c_void, f64, f64);
type CharFun = extern "C" fn(*mut c_void, u32);

type GetCurrentContext = extern "C" fn() -> *mut c_void;
type SetMouseButtonCallback = extern "C" fn(*mut c_void, MouseButtonFun) -> *mut c_void;
type SetCursorPosCallback = extern "C" fn(*mut c_void, CursorPosFun) -> *mut c_void;
type SetKeyCallback = extern "C" fn(*mut c_void, KeyFun) -> *mut c_void;
type SetScrollCallback = extern "C" fn(*mut c_void, ScrollFun) -> *mut c_void;
type SetCharCallback = extern "C" fn(*mut c_void, CharFun) -> *mut c_void;
type SetInputMode = extern "C" fn(*mut c_void, i32, i32);
type SetCursorPos = extern "C" fn(*mut c_void, f64, f64);

struct GlfwHooks {
    library: Library,
    window: *mut c_void,
    original_mouse_button: *mut c_void,
    original_cursor_pos: *mut c_void,
    original_key: *mut c_void,
    original_scroll: *mut c_void,
    original_char: *mut c_void,
    set_input_mode: SetInputMode,
    set_cursor_pos: SetCursorPos,
}

unsafe impl Send for GlfwHooks {}
unsafe impl Sync for GlfwHooks {}

static HOOKS: OnceLock<GlfwHooks> = OnceLock::new();

static CURSOR_LOCK_X: AtomicU64 = AtomicU64::new(0);
static CURSOR_LOCK_Y: AtomicU64 = AtomicU64::new(0);

fn cursor_lock() -> (f64, f64) {
    (
        f64::from_bits(CURSOR_LOCK_X.load(Ordering::Relaxed)),
        f64::from_bits(CURSOR_LOCK_Y.load(Ordering::Relaxed)),
    )
}

fn set_cursor_lock(x: f64, y: f64) {
    CURSOR_LOCK_X.store(x.to_bits(), Ordering::Relaxed);
    CURSOR_LOCK_Y.store(y.to_bits(), Ordering::Relaxed);
}

fn terminal_open() -> bool {
    crate::module::client::terminal::TerminalModule::is_active()
}

fn overlay_open() -> bool {
    GUI_OPEN.load(Ordering::Relaxed) || terminal_open()
}

extern "C" fn on_mouse_button(window: *mut c_void, button: i32, action: i32, mods: i32) {
    if let Ok(mut state) = MOUSE_STATE.lock() {
        match (button, action) {
            (GLFW_MOUSE_BUTTON_LEFT, GLFW_PRESS) => {
                state.left_down = true;
                state.left_clicked = true;
            }
            (GLFW_MOUSE_BUTTON_LEFT, GLFW_RELEASE) => state.left_down = false,
            (GLFW_MOUSE_BUTTON_RIGHT, GLFW_PRESS) => {
                state.right_down = true;
                state.right_clicked = true;
            }
            (GLFW_MOUSE_BUTTON_RIGHT, GLFW_RELEASE) => state.right_down = false,
            (GLFW_MOUSE_BUTTON_MIDDLE, GLFW_PRESS) if !overlay_open() => {
                state.middle_clicked = true;
            }
            _ => {}
        }
    }

    if overlay_open() {
        return;
    }

    if let Some(hooks) = HOOKS.get() {
        if !hooks.original_mouse_button.is_null() {
            let original: MouseButtonFun =
                unsafe { std::mem::transmute(hooks.original_mouse_button) };
            original(window, button, action, mods);
        }
    }
}

pub fn take_middle_click() -> bool {
    if overlay_open() {
        if let Ok(mut state) = MOUSE_STATE.lock() {
            state.middle_clicked = false;
        }
        return false;
    }
    MOUSE_STATE
        .lock()
        .map(|mut state| std::mem::take(&mut state.middle_clicked))
        .unwrap_or(false)
}

extern "C" fn on_cursor_pos(window: *mut c_void, x: f64, y: f64) {
    if let Ok(mut state) = MOUSE_STATE.lock() {
        state.x = x;
        state.y = y;
    }

    let gui_open = GUI_OPEN.load(Ordering::Relaxed);
    let terminal_open = terminal_open();
    let (send_x, send_y) = if gui_open {
        cursor_lock()
    } else if terminal_open {
        (x, y)
    } else {
        set_cursor_lock(x, y);
        (x, y)
    };

    if terminal_open && !gui_open {
        return;
    }

    if let Some(hooks) = HOOKS.get() {
        if !hooks.original_cursor_pos.is_null() {
            let original: CursorPosFun = unsafe { std::mem::transmute(hooks.original_cursor_pos) };
            original(window, send_x, send_y);
        }
    }
}

extern "C" fn on_key(window: *mut c_void, key: i32, scancode: i32, action: i32, mods: i32) {
    if action == GLFW_PRESS {
        LAST_KEY_PRESSED.store(key, Ordering::Relaxed);
    }

    let gui_was_open = GUI_OPEN.load(Ordering::Relaxed);
    let overlay_was_open = gui_was_open || terminal_open();

    if action == GLFW_PRESS
        && !GUI_KEY_CAPTURE_ACTIVE.load(Ordering::Relaxed)
        && (key == crate::config::gui_open_key() || (key == GLFW_KEY_ESCAPE && gui_was_open))
    {
        toggle_gui();
    }

    if overlay_was_open {
        if let Some(egui_key) = translate_glfw_key(key) {
            let pressed = action != GLFW_RELEASE;
            push_event(egui::Event::Key {
                key: egui_key,
                physical_key: None,
                pressed,
                repeat: action == GLFW_REPEAT,
                modifiers: translate_mods(mods),
            });
        }
    }

    if overlay_was_open || overlay_open() {
        return;
    }

    if action == GLFW_PRESS {
        handle_module_keybind(key);
    }

    if let Some(hooks) = HOOKS.get() {
        if !hooks.original_key.is_null() {
            let original: KeyFun = unsafe { std::mem::transmute(hooks.original_key) };
            original(window, key, scancode, action, mods);
        }
    }
}

extern "C" fn on_char(window: *mut c_void, codepoint: u32) {
    let overlay_was_open = overlay_open();

    if overlay_was_open {
        if let Some(ch) = char::from_u32(codepoint) {
            push_event(egui::Event::Text(ch.to_string()));
        }
    }

    if overlay_was_open || overlay_open() {
        return;
    }

    if let Some(hooks) = HOOKS.get() {
        if !hooks.original_char.is_null() {
            let original: CharFun = unsafe { std::mem::transmute(hooks.original_char) };
            original(window, codepoint);
        }
    }
}

extern "C" fn on_scroll(window: *mut c_void, x_offset: f64, y_offset: f64) {
    if overlay_open() {
        if let Ok(mut state) = MOUSE_STATE.lock() {
            state.scroll_x += x_offset as f32;
            state.scroll_y += y_offset as f32;
        }
        return;
    }

    if let Some(hooks) = HOOKS.get() {
        if !hooks.original_scroll.is_null() {
            let original: ScrollFun = unsafe { std::mem::transmute(hooks.original_scroll) };
            original(window, x_offset, y_offset);
        }
    }
}

pub fn begin_gui_key_capture() {
    LAST_KEY_PRESSED.store(-1, Ordering::Relaxed);
    GUI_KEY_CAPTURE_ACTIVE.store(true, Ordering::Relaxed);
}

pub fn finish_gui_key_capture() {
    GUI_KEY_CAPTURE_ACTIVE.store(false, Ordering::Relaxed);
}

fn toggle_gui() {
    let open = !GUI_OPEN.fetch_xor(true, Ordering::Relaxed);

    if !open {
        close_terminal_with_gui();
        crate::config::save();
    }

    let Some(hooks) = HOOKS.get() else {
        return;
    };
    if hooks.window.is_null() {
        return;
    }

    if open {
        (hooks.set_input_mode)(hooks.window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
    } else if terminal_open() {
        (hooks.set_input_mode)(hooks.window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
    } else if minecraft_screen_open() {
        (hooks.set_input_mode)(hooks.window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
    } else {
        let (lock_x, lock_y) = cursor_lock();
        (hooks.set_cursor_pos)(hooks.window, lock_x, lock_y);
        (hooks.set_input_mode)(hooks.window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    }
}

fn close_terminal_with_gui() {
    let Some(module) = client().modules.get(ModuleId::Terminal) else {
        return;
    };
    let Ok(mut module) = module.lock() else {
        log::error!("terminal: module lock poisoned while closing the GUI");
        return;
    };
    if !module.get_module_data().enabled {
        return;
    }
    if client().modules.is_active() {
        if let Err(error) = module.on_stop() {
            log::error!("terminal: failed to stop while closing the GUI: {error}");
        }
    }
    module.get_module_data_mut().set_enabled(false);
}

pub fn set_terminal_cursor(open: bool) {
    if GUI_OPEN.load(Ordering::Relaxed) {
        return;
    }
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    if hooks.window.is_null() {
        return;
    }
    if open || minecraft_screen_open() {
        (hooks.set_input_mode)(hooks.window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
    } else {
        let (lock_x, lock_y) = cursor_lock();
        (hooks.set_cursor_pos)(hooks.window, lock_x, lock_y);
        (hooks.set_input_mode)(hooks.window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    }
}

fn handle_module_keybind(key: i32) {
    let minecraft = minecraft();
    if !minecraft.current_screen_is_null() || !minecraft.in_world() {
        return;
    }

    let mut changed = false;
    for handle in client().modules.handles() {
        let Ok(mut module) = handle.lock() else {
            continue;
        };
        if module.get_module_data().key_bind as i32 != key {
            continue;
        }

        let enabled = !module.get_module_data().enabled;
        info!(
            "{} {}",
            module.get_module_data().name(),
            if enabled { "enabled" } else { "disabled" }
        );
        if enabled {
            let _ = module.on_start();
        } else {
            let _ = module.on_stop();
        }
        module.get_module_data_mut().set_enabled(enabled);
        changed = true;
    }

    if changed {
        crate::config::save();
    }
}

pub fn init() {
    if HOOKS.get().is_some() {
        return;
    }
    if let Some(hooks) = install_glfw_hooks() {
        let _ = HOOKS.set(hooks);
    }
}

fn install_glfw_hooks() -> Option<GlfwHooks> {
    let library = crate::graphic::platform::open_glfw_library()?;

    unsafe {
        let get_context = *library
            .get::<GetCurrentContext>(b"glfwGetCurrentContext")
            .ok()?;
        let set_mouse_button = *library
            .get::<SetMouseButtonCallback>(b"glfwSetMouseButtonCallback")
            .ok()?;
        let set_cursor_pos_cb = *library
            .get::<SetCursorPosCallback>(b"glfwSetCursorPosCallback")
            .ok()?;
        let set_key = *library.get::<SetKeyCallback>(b"glfwSetKeyCallback").ok()?;
        let set_scroll = *library
            .get::<SetScrollCallback>(b"glfwSetScrollCallback")
            .ok()?;
        let set_char = *library
            .get::<SetCharCallback>(b"glfwSetCharCallback")
            .ok()?;
        let set_input_mode = *library.get::<SetInputMode>(b"glfwSetInputMode").ok()?;
        let set_cursor_pos = *library.get::<SetCursorPos>(b"glfwSetCursorPos").ok()?;

        let window = get_context();
        if window.is_null() {
            return None;
        }

        info!("GLFW window acquired; installing input callbacks.");

        let original_mouse_button = set_mouse_button(window, on_mouse_button);
        let original_cursor_pos = set_cursor_pos_cb(window, on_cursor_pos);
        let original_key = set_key(window, on_key);
        let original_scroll = set_scroll(window, on_scroll);
        let original_char = set_char(window, on_char);

        if GUI_OPEN.load(Ordering::Relaxed) || terminal_open() {
            set_input_mode(window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
        }

        Some(GlfwHooks {
            library,
            window,
            original_mouse_button,
            original_cursor_pos,
            original_key,
            original_scroll,
            original_char,
            set_input_mode,
            set_cursor_pos,
        })
    }
}

pub fn cleanup() {
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    if hooks.window.is_null() {
        return;
    }

    type RestoreCallback = extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
    let restorations: [(&[u8], *mut c_void); 5] = [
        (b"glfwSetMouseButtonCallback", hooks.original_mouse_button),
        (b"glfwSetCursorPosCallback", hooks.original_cursor_pos),
        (b"glfwSetKeyCallback", hooks.original_key),
        (b"glfwSetScrollCallback", hooks.original_scroll),
        (b"glfwSetCharCallback", hooks.original_char),
    ];
    unsafe {
        for (name, original) in restorations {
            if let Ok(restore) = hooks.library.get::<RestoreCallback>(name) {
                restore(hooks.window, original);
            }
        }
    }

    let cursor_mode = if minecraft_screen_open() {
        GLFW_CURSOR_NORMAL
    } else {
        GLFW_CURSOR_DISABLED
    };
    (hooks.set_input_mode)(hooks.window, GLFW_CURSOR, cursor_mode);
    info!("GLFW input callbacks restored.");
}

fn minecraft_screen_open() -> bool {
    !minecraft().current_screen_is_null()
}
