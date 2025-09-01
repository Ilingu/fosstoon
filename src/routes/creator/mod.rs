use leptos::{prelude::*, task::spawn_local};
use leptos_meta::Style;
use leptos_router::{
    hooks::{use_navigate, use_params},
    params::Params,
};

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{
    components::{waiting_screen::WaitingScreen, webtoon::Webtoon},
    parse_or_navigate,
    utility::types::{Alert, AlertLevel, CreatorInfo, DownloadState, Language, WebtoonSearchInfo},
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Params, PartialEq, Debug, Clone)]
struct CreatorParams {
    id: Option<String>,
}

#[derive(Serialize)]
struct FetchCreatorArgs {
    profile_id: String,
    language: Language,
}

#[component]
pub fn CreatorPage() -> impl IntoView {
    /* url params */
    let params_args = use_params::<CreatorParams>();

    /* context */
    let push_toast =
        use_context::<Callback<Alert>>().expect("expected a 'set_alerts' context provided");

    /* states */
    let (creator_data, set_creator_data) = signal(None::<CreatorInfo>);

    /* Handlers */
    let fetch_creator_data = move |aid: String| {
        let navigate = use_navigate();
        spawn_local(async move {
            // fetch creator
            let creator_data = parse_or_navigate!(
                invoke(
                    "get_author_info",
                    serde_wasm_bindgen::to_value(&FetchCreatorArgs {
                        profile_id: aid,
                        language: Language::default()
                    })
                    .unwrap()
                )
                .await,
                Ty = CreatorInfo,
                push_toast,
                navigate,
                "/"
            );
            set_creator_data.set(Some(creator_data));
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
                view! {
                    <WaitingScreen dl_state=RwSignal::new(DownloadState::WebtoonData(0))
                        .split()
                        .0 />
                }
            }
        >
            <div id="creator_page">
                <h1 class="author_name">{move || creator_data.get().unwrap().name}</h1>
                <p class="followers">
                    {move || creator_data.get().unwrap().followers.unwrap_or_default()} " followers"
                </p>

                <div id="webtoons">
                    <For
                        each=move || creator_data.get().unwrap().webtoons
                        key=|wt| wt.id.wt_id
                        let(wt: WebtoonSearchInfo)
                    >
                        <Webtoon wt_info=wt.clone() is_local=false />
                    </For>
                </div>
            </div>
        </Show>
    }
}
