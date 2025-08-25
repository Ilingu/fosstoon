mod app;
mod components;
mod routes;
mod utility;

use app::*;
use leptos::prelude::*;

// for the different states: https://book.leptos.dev/15_global_state.html#option-3-create-a-global-state-store

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}
