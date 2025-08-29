use crate::{
    components::alert::Alert,
    routes::home::Home,
    utility::{
        store::{LoadingState, UserData},
        types::Alert,
    },
};

use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use reactive_stores::Store;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke, catch)]
    async fn invoke_without_args(cmd: &str) -> Result<JsValue, JsValue>;
}

#[component]
pub fn App() -> impl IntoView {
    /* App store */
    let user_state = Store::new(UserData::default());
    provide_context(user_state);

    let user_data_resp = LocalResource::new(move || invoke_without_args("get_user_data"));

    Effect::new(move |_| {
        let user_data = match user_data_resp.get().map(|opv| {
            opv.map(|v| {
                serde_wasm_bindgen::from_value::<UserData>(v)
                    .map_err(|_| "Failed to parse data as the right struct".to_string())
            })
            .map_err(|e| {
                e.as_string().unwrap_or(
                    "An error happened, but we can't provide more information".to_string(),
                )
            })
        }) {
            Some(Ok(Ok(mut us))) => {
                us.loading_state = LoadingState::Completed;
                us
            }
            None => return,
            // on error case, maybe do a user_data_resp.refetch() ?
            Some(Err(e)) | Some(Ok(Err(e))) => UserData {
                loading_state: LoadingState::Error(e),
                ..Default::default()
            },
        };

        user_state.set(user_data);
    });

    /* Alert system */
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
