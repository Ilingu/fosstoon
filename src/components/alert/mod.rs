use leptos::prelude::*;
use leptos_meta::Style;

use crate::utility::types::Alert;

#[component]
pub fn Alert(alert: Alert) -> impl IntoView {
    view! {
    <Style>{include_str!("alert.css")}</Style>
    <div class="alert">{alert.msg}</div> }
}
