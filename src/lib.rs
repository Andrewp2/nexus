use cfg_if::cfg_if;
pub mod app;
#[cfg(feature = "ssr")]
pub mod app_state;
pub mod dynamo;
pub mod error_template;
pub mod errors;
pub mod fileserv;
pub mod server;
pub mod site;

cfg_if! { if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::app::*;

    #[wasm_bindgen]
    pub fn hydrate() {
        // initializes logging using the `log` crate
        _ = console_log::init_with_level(log::Level::Debug);
        console_error_panic_hook::set_once();

        leptos::mount_to_body(App);
    }
}}

