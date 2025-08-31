use leptos::task::spawn_local;
use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_meta::Style;
use leptos_router::{
    hooks::{use_navigate, use_query},
    params::Params,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use icondata as i;
use leptos_icons::Icon;

use crate::utility::types::{Alert, AlertLevel, WebtoonId, WebtoonInfo, WtType};
use crate::{parse_or_navigate, parse_or_toast};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct FetchWtInfoArgs {
    id: WebtoonId,
}

#[derive(Params, PartialEq, Debug, Clone)]
struct WebtoonQueryArgs {
    wt_id: Option<usize>,
    wt_type: Option<WtType>,
}

// hazeshift: 7576

#[derive(Debug, Clone, Deserialize)]
enum WtDownloadingInfo {
    WebtoonData(u8),
    EpisodeInfo(u8),
    CachingImages(u8),

    Idle,
}

impl WtDownloadingInfo {
    fn get_progress(&self) -> u8 {
        *match self {
            WtDownloadingInfo::WebtoonData(p)
            | WtDownloadingInfo::CachingImages(p)
            | WtDownloadingInfo::EpisodeInfo(p) => p,
            WtDownloadingInfo::Idle => &0_u8,
        }
    }
    fn get_state(&self) -> String {
        match self {
            WtDownloadingInfo::WebtoonData(_) => "Fetching webtoon informations...",
            WtDownloadingInfo::EpisodeInfo(_) => "Fecthing episodes informations...",
            WtDownloadingInfo::CachingImages(_) => "Caching thumbnail (may take a while)...",
            WtDownloadingInfo::Idle => "Currently not downloading anything",
        }
        .to_string()
    }
}

#[component]
pub fn WebtoonPage() -> impl IntoView {
    let raw_wt_query_args = use_query::<WebtoonQueryArgs>();

    let push_toast =
        use_context::<Callback<Alert>>().expect("expected a 'set_alerts' context provided");

    let (webtoon_info, set_wt_info) = signal(None::<WebtoonInfo>);
    let (downloading_state, set_dl_state) = signal(WtDownloadingInfo::Idle);

    let fetch_wt_info = move |webtoon_id: WebtoonId| {
        let navigate = use_navigate();

        spawn_local(async move {
            let closure = Closure::<dyn FnMut(_)>::new(move |jsv: JsValue| {
                #[derive(Deserialize)]
                struct Event {
                    payload: WtDownloadingInfo,
                }

                if let Ok(Event { payload: dl_info }) = serde_wasm_bindgen::from_value::<Event>(jsv)
                {
                    console_log(&format!("event info: {:?}", dl_info));
                    set_dl_state.set(dl_info);
                };
            });
            listen("wt_dl_channel", closure.as_ref().unchecked_ref()).await;

            // fetch
            let wt_info = parse_or_navigate!(
                invoke(
                    "get_webtoon_info",
                    serde_wasm_bindgen::to_value(&FetchWtInfoArgs { id: webtoon_id }).unwrap()
                )
                .await,
                Ty = WebtoonInfo,
                push_toast,
                navigate,
                "/"
            );
            console_log(&format!("{wt_info:?}"));
            set_wt_info.set(Some(wt_info));

            closure.forget();
        });
    };

    Effect::new(move |_| {
        let navigate = use_navigate();

        match raw_wt_query_args.get() {
            Ok(WebtoonQueryArgs {
                wt_id: Some(wt_id),
                wt_type: Some(wt_type),
            }) => {
                let webtoon_id = WebtoonId::new(wt_id, wt_type);
                fetch_wt_info(webtoon_id);
            }
            Ok(WebtoonQueryArgs { wt_id: None, .. }) => {
                push_toast.run(Alert::new(
                    "Invalid webtoon id, returning home...",
                    AlertLevel::Warning,
                    None,
                ));
                navigate("/", Default::default());
            }
            Ok(WebtoonQueryArgs { wt_type: None, .. }) => {
                push_toast.run(Alert::new(
                    "Invalid webtoon type, returning home...",
                    AlertLevel::Warning,
                    None,
                ));
                navigate("/", Default::default());
            }
            Err(e) => {
                push_toast.run(Alert::new(
                    &format!("Failed to parse webtoon identifier, returning home: {e:?}"),
                    AlertLevel::Warning,
                    None,
                ));
                navigate("/", Default::default());
            }
        };
    });

    view! {
        <Style>{include_str!("wt.css")}</Style>
        <Show
            when=move || { webtoon_info.get().is_some() }
            fallback=move || view! { <WaitingScreen downloading_state /> }
        >
            <div class=""></div>
        </Show>
    }
}

#[component]
fn WaitingScreen(downloading_state: ReadSignal<WtDownloadingInfo>) -> impl IntoView {
    view! {
        <div id="loading_screen">
            <Icon icon=i::CgSpinnerAlt />
            <progress max="100" value=move || downloading_state.get().get_progress() />
            <p>{move || downloading_state.get().get_state()}</p>
        </div>
    }
}
