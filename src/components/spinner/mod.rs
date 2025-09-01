use leptos::prelude::*;
use leptos_meta::Style;

use icondata as i;
use leptos_icons::Icon;

#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <Style>{include_str!("spinner.css")}</Style>
        <Icon icon=i::CgSpinnerAlt />
    }
}
