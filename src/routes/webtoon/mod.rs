use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Style;
use leptos_router::{
    hooks::{use_navigate, use_query},
    params::Params,
};
use reactive_stores::Store;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use icondata as i;
use leptos_icons::Icon;

use crate::components::waiting_screen::WaitingScreen;
use crate::utility::convert_file_src;
use crate::utility::store::{LoadingState, UserData, UserDataStoreFields};
use crate::utility::types::{
    Alert, AlertLevel, DownloadState, EpisodePreview, Schedule, WebtoonId, WebtoonInfo, WtType,
};
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

#[derive(Serialize, Deserialize)]
struct SubWtInfoArgs {
    webtoon_id: WebtoonId,
}

#[derive(Params, PartialEq, Debug, Clone)]
struct WebtoonQueryArgs {
    wt_id: Option<usize>,
    wt_type: Option<WtType>,
}

#[derive(Debug, Clone)]
enum EpOrder {
    Latest,
    Oldest,
}

#[component]
pub fn WebtoonPage() -> impl IntoView {
    /* query */
    let raw_wt_query_args = use_query::<WebtoonQueryArgs>();

    /* context */
    let push_toast =
        use_context::<Callback<Alert>>().expect("expected a 'set_alerts' context provided");

    /* states */
    let user_state = expect_context::<Store<UserData>>();

    let (webtoon_info, set_wt_info) = signal(None::<WebtoonInfo>);
    let (dl_state, set_dl_state) = signal(DownloadState::Idle);
    let (ep_order, set_ep_order) = signal(EpOrder::Latest);

    let is_subscribed = Memo::new(move |_| {
        webtoon_info
            .get()
            .map(|wt: WebtoonInfo| {
                user_state
                    .webtoons()
                    .get()
                    .contains_key(&wt.id.wt_id.to_string())
            })
            .unwrap_or_default()
    });

    let episodes = Memo::new(move |_| match webtoon_info.get() {
        Some(WebtoonInfo {
            episodes: Some(mut episodes),
            ..
        }) => {
            episodes.sort_by(|a, b| match ep_order.get() {
                EpOrder::Latest => b.number.cmp(&a.number),
                EpOrder::Oldest => a.number.cmp(&b.number),
            });
            Some(episodes)
        }
        _ => None,
    });

    /* Handlers */
    let toggle_ep_order = move |_| {
        set_ep_order.update(|ep_order| {
            *ep_order = match ep_order {
                EpOrder::Latest => EpOrder::Oldest,
                EpOrder::Oldest => EpOrder::Latest,
            }
        });
    };

    let toggle_sub = move |_| {
        if let Some(wt) = webtoon_info.get_untracked() {
            let is_sub = is_subscribed.get_untracked();

            let args = serde_wasm_bindgen::to_value(&SubWtInfoArgs { webtoon_id: wt.id }).unwrap();
            spawn_local(async move {
                let cmd_to_invoke = match is_sub {
                    true => "unsubscribe_from_webtoon",
                    false => "subscribe_to_webtoon",
                };

                match invoke(cmd_to_invoke, args).await {
                    Ok(_) => {
                        push_toast.run(Alert::new(
                            match is_sub {
                                true => "Unsubscribed successfully",
                                false => "Subscribed successfully",
                            },
                            AlertLevel::Success,
                            Some(Duration::from_secs(2)),
                        ));

                        // update user frontend store
                        match is_sub {
                            true => {
                                user_state.update(|us| {
                                    us.webtoons.remove(&wt.id.wt_id.to_string());
                                });
                            }
                            false => {
                                user_state.update(|us| {
                                    us.webtoons.insert(wt.id.wt_id.to_string(), wt.into());
                                });
                            }
                        };
                    }
                    Err(_) => push_toast.run(Alert::new(
                        match is_sub {
                            true => "Failed to unsubscribe",
                            false => "Failed to subscribe",
                        },
                        AlertLevel::Error,
                        None,
                    )),
                }
            });
        }
    };
    let fetch_wt_info = move |webtoon_id: WebtoonId| {
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
            listen("wt_dl_channel", closure.as_ref().unchecked_ref()).await;

            // fetch webtoon
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
            set_wt_info.set(Some(wt_info));

            // close data gathering
            closure.forget();
        });
    };
    let force_ep_reload = move |_| {
        if let Some(wt) = webtoon_info.get_untracked() {
            spawn_local(async move {
                // refetch webtoon episodes
                let refreshed_wt = parse_or_toast!(
                    invoke(
                        "force_refresh_episodes",
                        serde_wasm_bindgen::to_value(&FetchWtInfoArgs { id: wt.id }).unwrap()
                    )
                    .await,
                    Ty = WebtoonInfo,
                    push_toast
                );
                set_wt_info.set(Some(refreshed_wt));
                push_toast.run(Alert::new(
                    "Episodes refreshed successfully",
                    AlertLevel::Success,
                    Some(Duration::from_secs(1)),
                ))
            });
        }
    };

    /* Effects */
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
            fallback=move || view! { <WaitingScreen dl_state /> }
        >
            <div id="webtoon_page">
                <header>
                    <a href="/">
                        <Icon icon=i::IoCaretBackOutline />
                    </a>
                    <button on:click=force_ep_reload>
                        <Icon icon=i::MdiReload />
                    </button>
                    <button on:click=toggle_sub>
                        <Show
                            when=move || {
                                user_state.loading_state().get() == LoadingState::Completed
                                    && is_subscribed.get()
                            }
                            fallback=move || view! { <Icon icon=i::BiBookmarkAltPlusSolid /> }
                        >
                            <Icon icon=i::BiBookmarkAltMinusSolid />
                        </Show>
                    </button>
                </header>
                <div id="wt_info">
                    <img
                        src=move || convert_file_src(&webtoon_info.get().unwrap().thumbnail)
                        class="wt_thumb"
                        alt="Webtoon thumbnail"
                    />
                    <p class="grades">
                        <span>{move || webtoon_info.get().unwrap().views} " views"</span>
                        <span>{move || webtoon_info.get().unwrap().subs} " subscribers"</span>
                    </p>
                    <p class="title">{move || webtoon_info.get().unwrap().title}</p>
                    <p class="creators">
                        {webtoon_info
                            .get()
                            .unwrap()
                            .creators
                            .into_iter()
                            .map(|c| match webtoon_info.get().unwrap().creator_id {
                                Some(aid) => {
                                    view! { <a href=format!("/creator/{aid}")>{c}</a> }.into_any()
                                }
                                None => view! { <span>{c}</span> }.into_any(),
                            })
                            .collect_view()}
                    </p>
                    <p class="summary">{move || webtoon_info.get().unwrap().summary}</p>
                    <Show when=move || webtoon_info.get().unwrap().schedule.is_some()>
                        <p class="schedule">
                            {move || match webtoon_info.get().unwrap().schedule.unwrap() {
                                Schedule::Daily => "Every DAY".to_string(),
                                Schedule::Completed => "COMPLETED".to_string(),
                                Schedule::Weekday(w) => format!("Every {w:?}"),
                                Schedule::Weekdays(w) => {
                                    let wd_str = w
                                        .iter()
                                        .map(|w| w.to_acronym())
                                        .collect::<Vec<_>>()
                                        .join(" | ");
                                    format!("Every {wd_str}")
                                }
                            }}
                        </p>
                    </Show>
                    <div class="genres">
                        {webtoon_info
                            .get()
                            .unwrap()
                            .genres
                            .into_iter()
                            .map(|g| view! { <div class="genre">{move || g.to_string()}</div> })
                            .collect_view()}
                    </div>

                    <Show when=move || episodes.get().is_some()>
                        <div id="episodes">
                            <header>
                                <p class="ep_count">
                                    {move || episodes.get().unwrap().len()} " episodes"
                                </p>
                                <button on:click=toggle_ep_order>
                                    {move || match ep_order.get() {
                                        EpOrder::Latest => {
                                            view! {
                                                <Icon icon=i::MdiSortNumericDescending />
                                                "latest"
                                            }
                                        }
                                        EpOrder::Oldest => {
                                            view! {
                                                <Icon icon=i::MdiSortNumericAscending />
                                                "oldest"
                                            }
                                        }
                                    }}
                                </button>
                            </header>

                            <For
                                each=move || episodes.get().unwrap()
                                key=|ep| ep.number
                                let(episode)
                            >
                                <Episode
                                    seen=match (webtoon_info.get(), is_subscribed.get()) {
                                        (Some(wt), true) => {
                                            user_state
                                                .webtoons()
                                                .get()
                                                .get(&wt.id.wt_id.to_string())
                                                .unwrap()
                                                .episode_seen
                                                .contains_key(&episode.number.to_string())
                                        }
                                        _ => false,
                                    }
                                    episode
                                />

                            </For>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn Episode(episode: EpisodePreview, seen: bool) -> impl IntoView {
    view! {
        <a
            href=move || {
                format!(
                    "/webtoon/episode/{}?wt_id={}&wt_type={}",
                    episode.number,
                    episode.parent_wt_id.wt_id,
                    episode.parent_wt_id.wt_type,
                )
            }
            class=format!("episode {}", if seen { "active" } else { "" })
        >
            <img src=move || convert_file_src(&episode.thumbnail) alt="Episode thumbnail" />
            <div class="ep_info">
                <p class="ep_title">{move || format!("#{} - {}", episode.number, episode.title)}</p>
                <p class="ep_date">{episode.posted_at}</p>
            </div>
            <p class="ep_likes">
                <Icon icon=i::AiHeartFilled />
                {episode.likes}
            </p>
        </a>
    }
}
