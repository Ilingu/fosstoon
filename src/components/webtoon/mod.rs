use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_meta::Style;

use crate::utility::types::WebtoonSearchInfo;

#[component]
pub fn Webtoon(
    #[prop(name = "wt_info")] WebtoonSearchInfo {
        title,
        thumbnail,
        creator,
        id,
    }: WebtoonSearchInfo,
) -> impl IntoView {
    view! {
        <Style>{include_str!("webtoon.css")}</Style>
        <div class="webtoon">
            <div class="thumbnail">
                <img src=thumbnail alt="Webtoon poster" />
            </div>
            <div class="title">
                <span>{title}</span>
            </div>
            <div class="type">
                <span>{id.wt_type.to_string()}</span>
            </div>
            <div class="author">
                <span>{creator}</span>
            </div>
        </div>
    }
}

#[component]
pub fn StandaloneWebtoon() -> impl IntoView {
    view! {
        <Style>{include_str!("webtoon.css")}</Style>
        <div class="webtoon standalone">
            <div class="thumbnail">
                <span></span>
            </div>
            <div class="title">
                <span></span>
            </div>
            <div class="type">
                <span></span>
            </div>
            <div class="author">
                <span></span>
            </div>
        </div>
    }
}

#[component]
pub fn GhostWebtoon() -> impl IntoView {
    view! {
        <Style>{include_str!("webtoon.css")}</Style>
        <div class="webtoon ghost"></div>
    }
}
