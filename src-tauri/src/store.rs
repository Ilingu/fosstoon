use serde::{Deserialize, Serialize};
use std::{collections::HashMap, default};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use webtoon::platform::webtoons::Language;

use crate::{
    constants::{USER_LANG_KEY, USER_STORE, USER_WEBTOONS_KEY, WEBTOONS_STORE},
    webtoon_handler::webtoon::{WebtoonId, WebtoonInfo},
};

/* TYPE DEF */

#[derive(Serialize, Deserialize, Clone)]
pub struct UserWebtoon {
    pub id: WebtoonId,
    pub title: String,
    pub thumbnail: Option<String>,
    pub last_seen: Option<usize>,
    pub episode_seen: HashMap<usize, bool>,
}

pub type UserWebtoons = HashMap<u32, UserWebtoon>;

#[derive(Default, Serialize, Deserialize)]
pub struct UserData {
    pub language: Language,
    pub webtoons: UserWebtoons,
}

#[derive(Serialize, Deserialize)]
pub struct WebtoonToStore {
    pub webtoon: WebtoonInfo,
    pub expire_at: usize,
}

/* IMPLEMENTATION */

impl From<WebtoonInfo> for UserWebtoon {
    fn from(
        WebtoonInfo {
            id,
            title,
            thumbnail,
            ..
        }: WebtoonInfo,
    ) -> Self {
        UserWebtoon {
            id,
            title,
            thumbnail,
            last_seen: None,
            episode_seen: HashMap::default(),
        }
    }
}

impl UserData {
    pub fn new(language: Language, webtoons: UserWebtoons) -> Self {
        Self { webtoons, language }
    }
}

/* COMMANDS */

// RULE: backend side has write/read access to the store. While frontend side only has read permission
#[tauri::command]
pub async fn change_language(
    user_state: tauri::State<'_, Mutex<UserData>>,
    app: tauri::AppHandle,
    new_lang: Language,
) -> Result<(), String> {
    // load ressources
    let user_store = app
        .store(USER_STORE)
        .map_err(|_| "Failed to open user store")?;
    let mut user_data = user_state.lock().await;

    // update stores accordingly
    user_store.set(
        USER_LANG_KEY,
        serde_json::to_value(new_lang).map_err(|_| "Couldn't serialize new_lang")?,
    );
    user_data.language = new_lang;

    Ok(())
}

/// TODO: if one of the store operation fails, you have to reverts the one before it to prevent merge conflict...
///
/// hmmm... logic may be implemented in client-side though
#[tauri::command]
pub async fn subscribe_to_webtoon(
    user_state: tauri::State<'_, Mutex<UserData>>,
    app: tauri::AppHandle,
    webtoon_id: WebtoonId,
) -> Result<WebtoonInfo, String> {
    // load ressources
    let user_store = app
        .store(USER_STORE)
        .map_err(|_| "Failed to open user store")?;
    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;
    let mut user_data = user_state.lock().await;

    // fetch webtoon data
    let mut webtoon2sub = WebtoonInfo::new_from_id(webtoon_id).await?;
    webtoon2sub.fetch_episodes().await?;

    // update user_state, user_store and webtoon_store accordingly
    user_data
        .webtoons
        .insert(webtoon_id.wt_id, webtoon2sub.clone().into());
    user_store.set(
        USER_WEBTOONS_KEY,
        serde_json::to_value(&user_data.webtoons)
            .map_err(|_| "Couldn't serialize user new webtoons")?,
    );
    webtoons_store.set(
        webtoon_id.wt_id.to_string(),
        serde_json::to_value(&webtoon2sub).map_err(|_| "Couldn't serialize webtoon2sub")?,
    );

    Ok(webtoon2sub)
}

/// TODO: if one of the store operation fails, you have to reverts the one before it to prevent merge conflict...
///
/// hmmm... logic may be implemented in client-side though
#[tauri::command]
pub async fn unsubscribe_from_webtoon(
    user_state: tauri::State<'_, Mutex<UserData>>,
    app: tauri::AppHandle,
    webtoon_id: WebtoonId,
) -> Result<(), String> {
    // load ressources
    let user_store = app
        .store(USER_STORE)
        .map_err(|_| "Failed to open user store")?;
    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;
    let mut user_data = user_state.lock().await;

    // update user_state, user_store and webtoon_store accordingly
    user_data.webtoons.remove(&webtoon_id.wt_id);
    user_store.set(
        USER_WEBTOONS_KEY,
        serde_json::to_value(&user_data.webtoons)
            .map_err(|_| "Couldn't serialize user new webtoons")?,
    );
    webtoons_store.delete(webtoon_id.wt_id.to_string());

    Ok(())
}
