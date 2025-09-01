use leptos::prelude::*;
use leptos_meta::Style;

use icondata as i;
use leptos_icons::Icon;

use crate::utility::types::{Alert, AlertLevel};

#[component]
pub fn Alert(alert: Alert) -> impl IntoView {
    view! {
        <Style>{include_str!("alert.css")}</Style>
        <div class=format!("alert {}", alert.level)>
            <span>
                {match alert.level {
                    AlertLevel::Success => {
                        view! { <Icon icon=i::BiCheckCircleRegular /> }.into_any()
                    }
                    AlertLevel::Info => view! { <Icon icon=i::AiInfoCircleOutlined /> }.into_any(),
                    AlertLevel::Warning => view! { <Icon icon=i::ImWarning /> }.into_any(),
                    AlertLevel::Error => view! { <Icon icon=i::ChOctagonWarning /> }.into_any(),
                }}
            </span>
            <p>{alert.msg}</p>
        </div>
    }
}
