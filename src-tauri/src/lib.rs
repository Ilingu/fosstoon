mod constants;
mod store;
mod webtoon_handler;

use crate::{
    constants::{USER_LANG_KEY, USER_STORE, USER_WEBTOONS_KEY},
    store::{UserData, UserWebtoons},
};

use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use webtoon::platform::webtoons::Language;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), String> {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // load user store
            let user_store = app.store(USER_STORE)?;

            let user_language = serde_json::from_value::<Language>(
                user_store.get(USER_LANG_KEY).unwrap_or_default(),
            )
            .unwrap_or_default();
            let user_webtoons = serde_json::from_value::<UserWebtoons>(
                user_store.get(USER_WEBTOONS_KEY).unwrap_or_default(),
            )
            .unwrap_or_default();

            let user_data: UserData = UserData::new(user_language, user_webtoons);

            // inject user store
            app.manage(Mutex::new(user_data));
            Ok(())
        })
        // .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .map_err(|_| "Failed to launch tauri app")?;
    Ok(())
}
