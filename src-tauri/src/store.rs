use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use webtoon::platform::webtoons::Language;

use crate::webtoon_handler::webtoon::{WebtoonId, WebtoonInfo};

#[derive(Serialize, Deserialize)]
pub struct UserWebtoon {
    pub id: String,
    pub title: String,
    pub last_seen: usize,
    pub episode_seen: HashMap<usize, bool>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct UserData {
    pub webtoons: HashMap<String, UserWebtoon>,
    pub language: Language,
}

#[derive(Serialize, Deserialize)]
pub struct WebtoonToStore {
    pub webtoon: WebtoonInfo,
    pub expire_at: usize,
}

// RULE: backend side has write/read access to the store. While frontend side only has read permission
#[tauri::command]
fn subscribe_to_webtoon(webtoon_id: WebtoonId) -> Result<(), String> {
    unimplemented!()
}

#[tauri::command]
fn unsubscribe_from_webtoon(webtoon_id: WebtoonId) -> Result<(), String> {
    unimplemented!()
}
