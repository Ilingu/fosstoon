use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Deref};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use webtoon::platform::webtoons::Language;
use webtoon_sdk::{webtoon::WebtoonInfo, WebtoonId};

use crate::constants::{USER_LANG_KEY, USER_STORE, USER_WEBTOONS_KEY, WEBTOONS_STORE};

/* TYPE DEF */

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserWebtoon {
    pub id: WebtoonId,
    pub title: String,
    pub thumbnail: String,
    pub creator: String,
    pub last_ep_num_seen: Option<usize>,
    pub episode_seen: HashMap<usize, bool>,
}

pub type UserWebtoons = HashMap<usize, UserWebtoon>;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserData {
    pub language: Language,
    pub webtoons: UserWebtoons,
}

/* IMPLEMENTATION */

impl From<WebtoonInfo> for UserWebtoon {
    fn from(
        WebtoonInfo {
            id,
            title,
            thumbnail,
            creators,
            ..
        }: WebtoonInfo,
    ) -> Self {
        UserWebtoon {
            id,
            title,
            thumbnail,
            creator: creators.first().cloned().unwrap_or_default(),
            last_ep_num_seen: None,
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

#[tauri::command]
pub async fn get_user_data(
    user_state: tauri::State<'_, Mutex<UserData>>,
) -> Result<UserData, String> {
    let user_data = user_state.lock().await;
    Ok(user_data.deref().clone())
}

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

#[tauri::command(rename_all = "snake_case")]
pub async fn subscribe_to_webtoon(
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

    // get webtoon data from storage
    let webtoon2sub = serde_json::from_value::<WebtoonInfo>(
        webtoons_store
            .get(webtoon_id.wt_id.to_string())
            .ok_or("No webtoons info found in storage")?,
    )
    .map_err(|e| e.to_string())?;

    // update user_state, user_store and webtoon_store accordingly
    user_data
        .webtoons
        .insert(webtoon_id.wt_id, webtoon2sub.clone().into());
    user_store.set(
        USER_WEBTOONS_KEY,
        serde_json::to_value(&user_data.webtoons)
            .map_err(|_| "Couldn't serialize user new webtoons")?,
    );

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn unsubscribe_from_webtoon(
    user_state: tauri::State<'_, Mutex<UserData>>,
    app: tauri::AppHandle,
    webtoon_id: WebtoonId,
) -> Result<(), String> {
    // load ressources
    let user_store = app
        .store(USER_STORE)
        .map_err(|_| "Failed to open user store")?;
    let mut user_data = user_state.lock().await;

    // update user_state, user_store and webtoon_store accordingly
    user_data.webtoons.remove(&webtoon_id.wt_id);
    user_store.set(
        USER_WEBTOONS_KEY,
        serde_json::to_value(&user_data.webtoons)
            .map_err(|_| "Couldn't serialize user new webtoons")?,
    );

    Ok(())
}
