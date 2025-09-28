use leptos::prelude::*;
use leptos_meta::Style;

use icondata as i;
use leptos_icons::Icon;
use leptos_router::components::A;

use crate::{components::spinner::Spinner, utility::types::DownloadState};

#[component]
pub fn WaitingScreen(dl_state: ReadSignal<DownloadState>) -> impl IntoView {
    view! {
        <Style>{include_str!("waiting_screen.css")}</Style>
        <div class="loading_screen">
            <div class="nav_back">
                <A href="/">
                    <Icon icon=i::IoCaretBackOutline />
                </A>
            </div>
            <Spinner />
            <progress max="100" value=move || dl_state.get().get_progress() />
            <p>{move || dl_state.get().get_state()}</p>
        </div>
    }
}
