use std::ffi::c_void;

use libloading::Library;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;
#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
compile_error!("the client supports Linux, Windows, and macOS only");

pub struct HookTarget {
    pub address: usize,

    pub label: String,
}

pub fn gl_proc_address(name: &str) -> *const c_void {
    imp::gl_proc_address(name)
}

pub fn open_glfw_library() -> Option<Library> {
    imp::open_glfw_library()
}

pub fn frame_hook_targets() -> Vec<HookTarget> {
    imp::frame_hook_targets()
}
