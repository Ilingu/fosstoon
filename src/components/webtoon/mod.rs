use leptos::prelude::*;
use leptos_meta::Style;

#[component]
pub fn Webtoon() -> impl IntoView {
    view! {
        <Style>{include_str!("webtoon.css")}</Style>
        <div></div>
    }
}

#[component]
pub fn StandaloneWebtoon() -> impl IntoView {
    view! {
        <Style>{include_str!("webtoon.css")}</Style>
        <div></div>
    }
}
