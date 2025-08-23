mod store;
mod webtoon_handler;

use crate::store::UserData;

use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // load user store
            let store = app.store("userstore.json")?;
            let user_webtoons =
                serde_json::from_value::<UserData>(store.get("user_webtoons").unwrap_or_default())
                    .unwrap_or_default();

            // inject user store
            app.manage(Mutex::new(user_webtoons));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        // .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
