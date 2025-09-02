use leptos::prelude::*;
use leptos_meta::Style;

use icondata as i;
use leptos_icons::Icon;

#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <Style>{include_str!("spinner.css")}</Style>
        <Icon
            icon=i::CgSpinnerAlt
            style="animation: spin 1.5s ease infinite; width: 2.2em; height: 2.2em;"
        />
    }
}
