/// Shadow println! when compiling to WASM
#[macro_export]
macro_rules! println {
        ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
    }

/// Shadow eprintln! when compiling to WASM
#[macro_export]
macro_rules! eprintln {
        ($($t:tt)*) => (web_sys::console::error_1(&format_args!($($t)*).to_string().into()))
    }

// TODO: Move WASM stuff to web server crate
// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
// pub async fn run() { init_logger(); }

/// We have to do a bit of setup to enable logging of panics.
///
/// "When wgpu hits any error, it panics with a generic message, while logging
/// the real error via the log crate. This means if you don't include
/// env_logger::init(), wgpu will fail silently, leaving you very confused!"
/// Reference: https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/#env-logger
pub fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        } else {
            env_logger::init();
        }
    }
}
