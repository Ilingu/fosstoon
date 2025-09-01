use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_meta::Style;
use leptos_router::{
    hooks::{use_navigate, use_params, use_query},
    params::Params,
};

#[component]
pub fn EpisodePage() -> impl IntoView {
    view! {
        <Style>{include_str!("episode.css")}</Style>
        <div></div>
    }
}
