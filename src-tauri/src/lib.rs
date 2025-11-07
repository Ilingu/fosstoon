mod constants;
mod store;
mod webtoon_handler;

use crate::{
    constants::{USER_LANG_KEY, USER_STORE, USER_WEBTOONS_KEY},
    store::{
        change_language, get_user_data, mark_as_read, subscribe_to_webtoon,
        unsubscribe_from_webtoon, UserData, UserWebtoons,
    },
    webtoon_handler::{
        creator::get_author_info,
        episodes::{force_refresh_episodes, get_episode_data, get_episode_post},
        webtoon::{
            delete_episodes, delete_webtoon, get_homepage_recommandations, get_webtoon_info,
            search_webtoon,
        },
    },
};

use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use webtoon::platform::webtoons::Language;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .invoke_handler(tauri::generate_handler![
            // stores
            get_user_data,
            subscribe_to_webtoon,
            unsubscribe_from_webtoon,
            mark_as_read,
            change_language,
            // webtoons
            search_webtoon,
            get_webtoon_info,
            get_homepage_recommandations,
            delete_episodes,
            delete_webtoon,
            // episodes
            get_episode_post,
            get_episode_data,
            force_refresh_episodes,
            // author
            get_author_info,
        ])
        .run(tauri::generate_context!())
        .expect("Failed to launch tauri app");
}
