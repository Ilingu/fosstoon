use leptos::prelude::*;
use leptos_meta::Style;

use crate::{components::spinner::Spinner, utility::types::DownloadState};

#[component]
pub fn WaitingScreen(dl_state: ReadSignal<DownloadState>) -> impl IntoView {
    view! {
        <Style>{include_str!("waiting_screen.css")}</Style>
        <div class="loading_screen">
            <Spinner />
            <progress max="100" value=move || dl_state.get().get_progress() />
            <p>{move || dl_state.get().get_state()}</p>
        </div>
    }
}
