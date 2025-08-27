use leptos::task::spawn_local;
use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_meta::Style;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::parse_or_toast;
use crate::utility::convert_file_src;
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

#[derive(Serialize, Deserialize)]
struct FetchWtImgArgs {
    images_url: Vec<String>,
    temporary_cache: bool,
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
    let search_webtoons = move || {
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
    let test_ft_wt_img = move |_| {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&FetchWtImgArgs {
                images_url: vec!["https://webtoon-phinf.pstatic.net/20250523_200/1748006227221ouWw5_JPEG/79690b17-1126-479b-bcb6-6bc5173b827c16195659268984580451.jpg?type=f160_151".to_string(), "https://webtoon-phinf.pstatic.net/20250715_138/1752541364726EKldt_JPEG/17525413646832751_S2ChasersEp13-05_003.jpg?type=q90".to_string()],
                temporary_cache: true,
            })
            .unwrap();

            let imgs_path = parse_or_toast!(
                invoke("fetch_wt_imgs", args).await,
                Ty = Vec<String>,
                push_toast
            );
            console_log(&format!("{imgs_path:?}"));
        });
    };
    // <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo" />

    view! {
        <Style>{include_str!("home.css")}</Style>
        <main class="container">
            <img src=move || convert_file_src("/data/user/0/com.ilingu.fosstoon/cache/17525413646832751_S2ChasersEp13-05_003.jpg") alt="panel" />
            <button on:click=test_ft_wt_img>"Search"</button>
        </main>
    }
}
