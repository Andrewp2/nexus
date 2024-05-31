#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();

    // initializes logging using the `log` crate
    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount_to_body(NexusApp);
}
