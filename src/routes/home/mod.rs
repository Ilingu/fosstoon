use std::fmt::Display;
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Style;

use icondata as i;
use leptos_icons::Icon;

use reactive_stores::Store;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::components::webtoon::{StandaloneWebtoon, Webtoon};
use crate::parse_or_toast;
use crate::utility::store::{
    LoadingState, UserData, UserDataStoreFields, UserRecommendations,
    UserRecommendationsStoreFields, UserWebtoon,
};
use crate::utility::types::{Alert, AlertLevel, WebtoonId, WebtoonSearchInfo};

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
struct FetchWtInfoArgs {
    id: WebtoonId,
}

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    My,
    Recommandation,
    Search(RwSignal<String>),
}

impl Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppMode::My => "my",
                AppMode::Recommandation => "recommandation",
                AppMode::Search(_) => "search",
            }
        )
    }
}

#[component]
pub fn Home() -> impl IntoView {
    /* Global state */
    let user_state = expect_context::<Store<UserData>>();
    let user_rec_state = expect_context::<Store<UserRecommendations>>();
    let push_toast =
        use_context::<Callback<Alert>>().expect("expected a 'set_alerts' context provided");

    let user_webtoons = Memo::new(move |_| {
        let mut uwt = user_state
            .webtoons()
            .get()
            .into_values()
            .collect::<Vec<UserWebtoon>>();
        uwt.sort_by_key(|uwt| uwt.last_ep_num_seen);
        uwt.into_iter()
            .map(|uwt| uwt.into())
            .collect::<Vec<WebtoonSearchInfo>>()
    });

    /* states */
    let (webtoons, set_webtoons) = signal::<Vec<WebtoonSearchInfo>>(vec![]);
    let (app_mode, set_app_mode) = signal(AppMode::My);

    /* handlers */
    let load_user_wt = move || {
        if user_state.loading_state().get_untracked() == LoadingState::Completed
            && app_mode.get_untracked() == AppMode::My
        {
            set_webtoons.set(user_webtoons.get_untracked());
        }
    };

    let load_user_rec = move || {
        if user_rec_state.loading_state().get_untracked() == LoadingState::Completed
            && app_mode.get_untracked() == AppMode::Recommandation
        {
            let recommendations = user_rec_state.webtoons().get_untracked();
            set_webtoons.set(recommendations);
        }
    };

    let search_webtoons = move || {
        if let AppMode::Search(query) = app_mode.get_untracked() {
            let query = query.get_untracked().trim().to_string();
            if query.is_empty() || query.len() <= 2 {
                return;
            }
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&SearchWtArgs { query: &query }).unwrap();

                push_toast.run(Alert::new(
                    "Searching...",
                    AlertLevel::Info,
                    Some(Duration::from_millis(400)),
                ));
                let webtoons = parse_or_toast!(
                    invoke("search_webtoon", args).await,
                    Ty = Vec<WebtoonSearchInfo>,
                    push_toast
                );
                set_webtoons.set(webtoons);
            });
        }
    };

    /* Effects */
    Effect::new(move |_| match user_state.loading_state().get() {
        LoadingState::Loading => (),
        LoadingState::Completed => load_user_wt(),
        LoadingState::Error(e) => {
            set_app_mode.set(AppMode::Recommandation);
            push_toast.run(Alert::new(&e, AlertLevel::Error, None));
        }
    });

    Effect::new(move |_| match user_rec_state.loading_state().get() {
        LoadingState::Loading => (),
        LoadingState::Completed => {
            push_toast.run(Alert::new(
                "Recommendations loaded",
                AlertLevel::Info,
                Some(Duration::from_millis(300)),
            ));
            if let AppMode::Recommandation = app_mode.get_untracked() {
                load_user_rec();
            }
        }
        LoadingState::Error(e) => {
            push_toast.run(Alert::new(&e, AlertLevel::Error, None));
        }
    });

    let search_timeout = StoredValue::new(None::<TimeoutHandle>);
    let before_search_app_mode = StoredValue::new(AppMode::My);
    Effect::new(move || match app_mode.get() {
        AppMode::My => {
            before_search_app_mode.set_value(AppMode::My);
            load_user_wt();
        }
        AppMode::Recommandation => {
            before_search_app_mode.set_value(AppMode::Recommandation);
            load_user_rec();
        }
        AppMode::Search(q) => {
            // search one second after the user stopped typping
            if let Some(timeout_handle) = search_timeout.get_value() {
                timeout_handle.clear();
            }
            if q.get().trim().is_empty() {
                set_app_mode.set(before_search_app_mode.get_value());
                return;
            }

            let timeout_handle =
                set_timeout_with_handle(search_webtoons, Duration::from_millis(1500))
                    .expect("No timeout");
            search_timeout.set_value(Some(timeout_handle));
        }
    });

    // <img src=move || convert_file_src("/data/user/0/com.ilingu.fosstoon/cache/17525413646832751_S2ChasersEp13-05_003.jpg") />

    view! {
        <Style>{include_str!("home.css")}</Style>
        <main class="container">
            <div id="search">
                <img src="public/logo.png" class="logo" alt="Fosstoon logo" />
                <div class="search_input">
                    <input
                        type="text"

                        placeholder="🔍 Search webtoon"
                        on:input:target=move |ev| {
                            let val = ev.target().value();
                            if let AppMode::Search(q_sig) = app_mode.get_untracked() {
                                q_sig.set(val);
                            } else {
                                set_app_mode.set(AppMode::Search(RwSignal::new(val)));
                            }
                        }
                        prop:value=move || match app_mode.get() {
                            AppMode::My => RwSignal::new(String::new()),
                            AppMode::Recommandation => RwSignal::new(String::new()),
                            AppMode::Search(q_signal) => q_signal,
                        }
                    />
                    <button on:click=move |_| {
                        if let AppMode::Search(q_signal) = app_mode.get_untracked() {
                            q_signal.set(String::new());
                        }
                    }>
                        <Icon icon=i::ChCircleCross />
                    </button>
                </div>

            </div>
            <div id="webtoons">
                <Show
                    when=move || {
                        !webtoons.get().is_empty()
                            || (app_mode.get() == AppMode::My
                                && user_state.loading_state().get() == LoadingState::Completed)
                            || (app_mode.get() == AppMode::Recommandation
                                && user_rec_state.loading_state().get() == LoadingState::Completed)
                    }
                    fallback=|| {
                        view! {
                            {(1..=10)
                                .map(|_| {
                                    view! { <StandaloneWebtoon /> }
                                })
                                .collect::<Vec<_>>()}
                        }
                    }
                >
                    <Show
                        when=move || { !webtoons.get().is_empty() }
                        fallback=|| {
                            view! { <p>"Nothing to show!"</p> }
                        }
                    >
                        <For
                            each=move || webtoons.get()
                            key=|wt| (wt.id.wt_id, wt.thumbnail.clone())
                            let(wt: WebtoonSearchInfo)
                        >
                            <Webtoon
                                wt_info=wt.clone()
                                is_local=app_mode.get() == AppMode::My
                                    || app_mode.get() == AppMode::Recommandation
                            />
                        </For>
                    </Show>

                </Show>
            </div>
            <nav>
                <button
                    on:click=move |_| set_app_mode.set(AppMode::My)
                    class=move || {
                        format!(
                            "btn {}",
                            match app_mode.get() {
                                AppMode::My => "active",
                                _ => "",
                            },
                        )
                    }
                >
                    <Icon icon=i::BiBookmarkHeartSolid />
                    "My"
                </button>
                <button
                    on:click=move |_| set_app_mode.set(AppMode::Recommandation)
                    class=move || {
                        format!(
                            "btn {}",
                            match app_mode.get() {
                                AppMode::Recommandation => "active",
                                _ => "",
                            },
                        )
                    }
                >
                    <Icon icon=i::BiWorldRegular />
                    "Explore"
                </button>
            </nav>
        </main>
    }
}
