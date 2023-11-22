mod components;
pub mod error_template;
mod model;
mod reactive_list;
mod utils;

use app::*;
use leptos::*;

pub mod app;
pub mod fileserv;

use crate::app::*;
use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    // initializes logging using the `log` crate
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}

