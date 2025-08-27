use crate::{components::alert::Alert, routes::home::Home, utility::types::Alert};

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    let (alerts, set_alerts) = signal::<Vec<Alert>>(vec![]);
    let push_toast = move |mut alert: Alert| {
        let a_id = match alerts.get_untracked().last() {
            Some(a) => a.id + 1,
            None => 0,
        };
        alert.id = a_id;
        set_alerts.write().push(alert.clone());
        set_timeout(
            move || set_alerts.write().retain(|a| a.id != a_id),
            alert.duration,
        )
    };
    provide_context(Callback::new(push_toast));

    view! {
        <Router>
            <Routes fallback=|| "Not found.">
                <Route path=path!("/") view=Home />
            </Routes>

            <div id="alerts">
                <For
                    each=move || alerts.get()
                    key=|state| state.id
                    children=move |alert| {
                        view! { <Alert alert /> }
                    }
                />
            </div>
        </Router>
    }
}
