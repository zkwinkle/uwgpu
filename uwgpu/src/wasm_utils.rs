pub use wasm_bindgen::prelude::*;

use winit::window::Window;

/// Appends the winit canvas to the 'wasm-canvas' id'd element in the page
pub fn wasm_add_canvas_to_html(window: &Window) {
    use winit::platform::web::WindowExtWebSys;

    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    use winit::dpi::PhysicalSize;
    let _ = window.request_inner_size(PhysicalSize::new(450, 400));

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("wasm-canvas")?;
            let canvas = web_sys::Element::from(window.canvas()?);
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

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
