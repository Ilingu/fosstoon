use leptos::{prelude::*, task::spawn_local};
use leptos_meta::Style;
use leptos_router::{
    hooks::{use_navigate, use_params},
    params::Params,
};

use icondata as i;
use leptos_icons::Icon;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    components::{waiting_screen::WaitingScreen, webtoon::Webtoon},
    parse_or_navigate,
    utility::types::{Alert, AlertLevel, DownloadState, WebtoonSearchInfo, WtCreator},
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[derive(Params, PartialEq, Debug, Clone)]
struct CreatorParams {
    id: Option<String>,
}

#[derive(Serialize)]
struct FetchCreatorArgs {
    profile_id: String,
}

#[component]
pub fn CreatorPage() -> impl IntoView {
    /* url params */
    let params_args = use_params::<CreatorParams>();

    /* context */
    let push_toast =
        use_context::<Callback<Alert>>().expect("expected a 'set_alerts' context provided");

    /* states */
    let (creator_data, set_creator_data) = signal(None::<WtCreator>);
    let (dl_state, set_dl_state) = signal(DownloadState::Idle);

    /* Handlers */
    let fetch_creator_data = move |aid: String| {
        let navigate = use_navigate();
        spawn_local(async move {
            // open data stream to get info of webtoon download progression
            let closure = Closure::<dyn FnMut(_)>::new(move |jsv: JsValue| {
                #[derive(Deserialize)]
                struct Event {
                    payload: DownloadState,
                }

                if let Ok(Event { payload: dl_info }) = serde_wasm_bindgen::from_value::<Event>(jsv)
                {
                    set_dl_state.set(dl_info);
                };
            });
            listen("creator_dl_channel", closure.as_ref().unchecked_ref()).await;

            // fetch creator
            let creator_data = parse_or_navigate!(
                invoke(
                    "get_author_info",
                    serde_wasm_bindgen::to_value(&FetchCreatorArgs { profile_id: aid }).unwrap()
                )
                .await,
                Ty = WtCreator,
                push_toast,
                navigate,
                "/"
            );
            set_creator_data.set(Some(creator_data));

            // close data gathering
            closure.forget();
        });
    };

    /* Effects */
    Effect::new(move |_| {
        let navigate = use_navigate();

        match params_args.get() {
            Ok(CreatorParams { id: Some(aid) }) => {
                set_creator_data.set(None);
                fetch_creator_data(aid);
            }
            _ => {
                push_toast.run(Alert::new(
                    "Failed to parse creator id, returning home",
                    AlertLevel::Warning,
                    None,
                ));
                navigate("/", Default::default());
            }
        };
    });

    // can directly show thumbnail, no block
    view! {
        <Style>{include_str!("creator.css")}</Style>
        <Show
            when=move || { creator_data.get().is_some() }
            fallback=move || {
                view! { <WaitingScreen dl_state /> }
            }
        >
            <div id="creator_page">
                <div class="nav_back">
                    <a href="/">
                        <Icon icon=i::IoCaretBackOutline />
                    </a>
                </div>
                <h1 class="author_name">{move || creator_data.get().unwrap().name}</h1>
                <p class="followers">
                    {move || creator_data.get().unwrap().followers} " followers"
                </p>

                <div id="webtoons">
                    <For
                        each=move || creator_data.get().unwrap().webtoons
                        key=|wt| wt.id.wt_id
                        let(wt: WebtoonSearchInfo)
                    >
                        <Webtoon wt_info=wt.clone() is_local=true />
                    </For>
                </div>
            </div>
        </Show>
    }
}
