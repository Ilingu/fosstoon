use std::{ops::Not, time::SystemTime};

use leptos::{prelude::*, task::spawn_local};
use leptos_meta::Style;
use leptos_router::{
    hooks::{use_navigate, use_params, use_query},
    params::Params,
};

use icondata as i;
use leptos_icons::Icon;

use reactive_stores::Store;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    components::waiting_screen::WaitingScreen,
    parse_or_navigate, parse_or_toast,
    utility::{
        convert_file_src,
        store::UserData,
        types::{Alert, AlertLevel, DownloadState, EpisodeData, Post, WebtoonId, WtType},
    },
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct EpIdArgs {
    wt_id: WebtoonId,
    ep_num: usize,
}

#[derive(Params, PartialEq, Debug, Clone)]
struct WebtoonQueryArgs {
    wt_id: Option<usize>,
    wt_type: Option<WtType>,
    prev_ep_read: Option<bool>,
}

#[derive(Params, PartialEq, Debug, Clone)]
struct EpisodeParams {
    num: Option<usize>,
}

#[component]
pub fn EpisodePage() -> impl IntoView {
    /* url params */
    let params_args = use_params::<EpisodeParams>();
    let query_args = use_query::<WebtoonQueryArgs>();

    /* context */
    let push_toast =
        use_context::<Callback<Alert>>().expect("expected a 'set_alerts' context provided");

    /* states */
    let user_state = expect_context::<Store<UserData>>();

    let (episode_data, set_episode_data) = signal(None::<EpisodeData>);
    let (ep_comments, set_ep_comments) = signal(None::<Vec<Post>>);
    let (dl_state, set_dl_state) = signal(DownloadState::Idle);

    /* Handlers */
    let fetch_ep_data = move |wt_id: WebtoonId, ep_num: usize| {
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
            listen("ep_dl_channel", closure.as_ref().unchecked_ref()).await;

            // fetch webtoon
            let ep_data = parse_or_navigate!(
                invoke(
                    "get_episode_data",
                    serde_wasm_bindgen::to_value(&EpIdArgs { wt_id, ep_num }).unwrap()
                )
                .await,
                Ty = EpisodeData,
                push_toast,
                navigate,
                &format!("/webtoon?wt_id={}&wt_type={}", wt_id.wt_id, wt_id.wt_type)
            );
            set_episode_data.set(Some(ep_data));

            // close data gathering
            closure.forget();
        });
    };

    let mark_prev_ep_as_read = move |wt_id: WebtoonId, current_ep: usize| {
        if current_ep <= 1 {
            return;
        }
        spawn_local(async move {
            parse_or_toast!(
                invoke(
                    "mark_as_read",
                    serde_wasm_bindgen::to_value(&EpIdArgs {
                        wt_id,
                        ep_num: current_ep - 1
                    })
                    .unwrap()
                )
                .await,
                Ty = (),
                push_toast
            );

            user_state.update(|us| {
                us.webtoons.entry(wt_id.wt_id.to_string()).and_modify(|wt| {
                    wt.episode_seen.insert((current_ep - 1).to_string(), true);
                    wt.last_seen = Some(SystemTime::now());
                });
            });
        });
    };

    let fetch_post = move |_| {
        if let Some(ep_data) = episode_data.get_untracked() {
            push_toast.run(Alert::new(
                "Posts are loading, it may take a while",
                AlertLevel::Info,
                None,
            ));
            spawn_local(async move {
                let posts = parse_or_toast!(
                    invoke(
                        "get_episode_post",
                        serde_wasm_bindgen::to_value(&EpIdArgs {
                            wt_id: ep_data.parent_wt_id,
                            ep_num: ep_data.number
                        })
                        .unwrap()
                    )
                    .await,
                    Ty = Vec<Post>,
                    push_toast
                );
                set_ep_comments.set(Some(posts));
            });
        }
    };

    /* Effects */
    Effect::new(move |_| {
        let navigate = use_navigate();

        match (query_args.get(), params_args.get()) {
            (
                Ok(WebtoonQueryArgs {
                    wt_id: Some(wt_id),
                    wt_type: Some(wt_type),
                    prev_ep_read,
                }),
                Ok(EpisodeParams { num: Some(ep_num) }),
            ) => {
                set_episode_data.set(None);
                set_ep_comments.set(None);

                let webtoon_id = WebtoonId::new(wt_id, wt_type);
                fetch_ep_data(webtoon_id, ep_num);
                if prev_ep_read.unwrap_or_default() {
                    mark_prev_ep_as_read(webtoon_id, ep_num);
                }
            }
            _ => {
                push_toast.run(Alert::new(
                    "Failed to parse identifier, returning home",
                    AlertLevel::Warning,
                    None,
                ));
                navigate("/", Default::default());
            }
        };
    });

    view! {
        <Style>{include_str!("episode.css")}</Style>
        <Show
            when=move || { episode_data.get().is_some() }
            fallback=move || view! { <WaitingScreen dl_state /> }
        >
            <div id="episode_page">
                <div id="panels">
                    <For
                        each=move || episode_data.get().unwrap().panels
                        key=|panel| panel.to_owned()
                        let(panel_url)
                    >
                        <img src=move || convert_file_src(&panel_url) alt="Episode panel" />
                    </For>
                </div>
                <div class="author_info">
                    <img
                        src=move || convert_file_src(&episode_data.get().unwrap().author_thumb)
                        alt="Author thumbnail"
                    />
                    <div>
                        {move || match episode_data.get().unwrap().author_id {
                            Some(aid) => {
                                view! {
                                    <a
                                        class="author_name"
                                        href=move || { format!("/creator/{aid}") }
                                    >
                                        {move || episode_data.get().unwrap().author_name}
                                    </a>
                                }
                                    .into_any()
                            }
                            None => {
                                view! {
                                    <p class="author_name">
                                        {move || episode_data.get().unwrap().author_name}
                                    </p>
                                }
                                    .into_any()
                            }
                        }}
                        <p class="author_note">{move || episode_data.get().unwrap().author_note}</p>
                    </div>
                </div>
                <div class="action">
                    <a href=move || {
                        let ep_data = episode_data.get().unwrap();
                        format!(
                            "/webtoon/episode/{}?wt_id={}&wt_type={}&prev_ep_read=true",
                            ep_data.number + 1,
                            ep_data.parent_wt_id.wt_id,
                            ep_data.parent_wt_id.wt_type,
                        )
                    }>
                        <div>
                            <p>"Next episode: "</p>
                            <p>"Episode " {move || episode_data.get().unwrap().number + 1}</p>
                        </div>
                        <Icon icon=i::AiCaretRightOutlined />
                    </a>
                </div>
                <div class="comments">
                    <h3>"Top Comments" <Icon icon=i::BiCommentDetailRegular /></h3>

                    <Show
                        when=move || ep_comments.get().is_some()
                        fallback=move || {
                            view! { <button on:click=fetch_post>"Click to fetch"</button> }
                        }
                    >
                        <For
                            each=move || ep_comments.get().unwrap()
                            key=|post| post.id.clone()
                            let(post_data)
                        >
                            <PostComponent post_data />
                        </For>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn PostComponent(post_data: Post) -> impl IntoView {
    let show_spoiler = RwSignal::new(false);
    let date = move || {
        // js_sys bindings calls JavaScript's standard lib
        js_sys::Date::new(&JsValue::from_f64(post_data.posted_at as f64))
            .to_date_string()
            .as_string()
    };

    view! {
        <div class="comment">
            <p class="poster">{post_data.poster_name}</p>
            <p class="date">{date}</p>
            <p class="content" on:click=move |_| show_spoiler.update(|s_sp| *s_sp = s_sp.not())>
                {move || match (post_data.is_spoiler, show_spoiler.get()) {
                    (true, true) | (false, _) => Some(post_data.content.to_owned()),
                    (true, false) => Some("**Click to show spoiler**".to_string()),
                }}
            </p>
            <div class="votes">
                <p>
                    <Icon icon=i::AiLikeFilled />
                    {post_data.upvotes}
                </p>
                <p>
                    <Icon icon=i::AiDislikeFilled />
                    {post_data.downvotes}
                </p>
            </div>
        </div>
    }
}
