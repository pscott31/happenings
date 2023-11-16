mod app;
mod components;
mod model;
mod reactive_list;
mod utils;

use app::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}

