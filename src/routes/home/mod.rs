use std::time::Duration;

use leptos::task::spawn_local;
use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_meta::Style;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::parse_or_toast;
use crate::utility::types::{Alert, AlertLevel, WebtoonSearchInfo};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize)]
struct SearchWtArgs<'a> {
    query: &'a str,
}

#[component]
pub fn Home() -> impl IntoView {
    let push_toast =
        use_context::<Callback<Alert>>().expect("expected a 'set_alerts' context provided");

    /* states */
    let (webtoons, set_webtoons) = signal::<Vec<WebtoonSearchInfo>>(vec![]);
    let (search_query, set_search_query) = signal("");

    let is_searching = move || !search_query.get().trim().is_empty();

    /* handlers */
    let search_webtoons = move |_| {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&SearchWtArgs {
                query: "7 first date",
            })
            .unwrap();

            let webtoons = parse_or_toast!(
                invoke("search_webtoon", args).await,
                Ty = Vec<WebtoonSearchInfo>,
                push_toast
            );
            console_log(&format!("{webtoons:?}"));
            *set_webtoons.write() = webtoons;
        });
    };
    // <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo" />

    view! {
        <Style>{include_str!("home.css")}</Style>
        <main class="container">
            <button on:click=search_webtoons>"Search"</button>
        </main>
    }
}
