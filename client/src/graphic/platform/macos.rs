use std::ffi::{c_void, CStr};
use std::path::Path;
use std::sync::OnceLock;

use libloading::Library;
use log::{info, warn};

use super::HookTarget;

const GL_LIBRARY: &str = "/System/Library/Frameworks/OpenGL.framework/Versions/Current/OpenGL";

fn gl_library() -> Option<&'static Library> {
    static LIB: OnceLock<Option<Library>> = OnceLock::new();
    // SAFETY: opening the system OpenGL framework.
    LIB.get_or_init(|| unsafe { Library::new(GL_LIBRARY).ok() })
        .as_ref()
}

/// Resolves an OpenGL function pointer by name.
pub fn gl_proc_address(name: &str) -> *const c_void {
    let Some(lib) = gl_library() else {
        return std::ptr::null();
    };
    // SAFETY: resolving an exported GL symbol by name.
    unsafe {
        match lib.get::<unsafe extern "system" fn()>(name.as_bytes()) {
            Ok(symbol) => *symbol as *const c_void,
            Err(_) => std::ptr::null(),
        }
    }
}

/// Opens the GLFW shared library the host loaded.
pub fn open_glfw_library() -> Option<Library> {
    loaded_glfw_library()
}

pub fn frame_hook_targets() -> Vec<HookTarget> {
    let Some(library) = loaded_glfw_library() else {
        warn!("could not find the loaded GLFW dylib; the overlay hook was not installed");
        return Vec::new();
    };

    // SAFETY: resolving an exported GLFW function from its loaded dylib.
    let address = unsafe {
        match library.get::<unsafe extern "system" fn()>(b"glfwSwapBuffers") {
            Ok(symbol) => *symbol as *const () as usize,
            Err(error) => {
                warn!("could not resolve glfwSwapBuffers: {error}");
                return Vec::new();
            }
        }
    };
    info!("found glfwSwapBuffers at 0x{address:x}");
    std::mem::forget(library);
    vec![HookTarget {
        address,
        label: "glfwSwapBuffers".to_string(),
    }]
}

fn loaded_glfw_library() -> Option<Library> {
    loaded_image_paths()
        .into_iter()
        .find(|path| {
            Path::new(path)
                .file_name()
                .is_some_and(|name| name.to_string_lossy().to_ascii_lowercase().contains("glfw"))
        })
        .and_then(|path| {
            // SAFETY: opening a GLFW dylib already loaded by the JVM.
            unsafe { Library::new(path).ok() }
        })
        .or_else(|| {
            // SAFETY: fallback names used by common LWJGL distributions.
            unsafe {
                Library::new("libglfw.3.dylib")
                    .or_else(|_| Library::new("libglfw.dylib"))
                    .ok()
            }
        })
}

fn loaded_image_paths() -> Vec<String> {
    let mut paths = Vec::new();
    // SAFETY: dyld returns names of images currently loaded in this process.
    unsafe {
        for index in 0.._dyld_image_count() {
            let name = _dyld_get_image_name(index);
            if !name.is_null() {
                paths.push(CStr::from_ptr(name).to_string_lossy().into_owned());
            }
        }
    }
    paths
}

extern "C" {
    fn _dyld_image_count() -> u32;
    fn _dyld_get_image_name(index: u32) -> *const std::ffi::c_char;
}
