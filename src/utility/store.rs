use std::{collections::HashMap, time::SystemTime};

use reactive_stores::Store;
use serde::{Deserialize, Serialize};

use crate::utility::types::{Language, WebtoonId, WebtoonInfo, WebtoonSearchInfo};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserWebtoon {
    pub id: WebtoonId,
    pub title: String,
    pub thumbnail: String,
    pub creator: String,
    pub last_seen: Option<SystemTime>,
    pub episode_seen: HashMap<String, bool>,
}

pub type UserWebtoons = HashMap<String, UserWebtoon>;

impl From<UserWebtoon> for WebtoonSearchInfo {
    fn from(
        UserWebtoon {
            id,
            title,
            thumbnail,
            creator,
            ..
        }: UserWebtoon,
    ) -> Self {
        WebtoonSearchInfo {
            id,
            title,
            thumbnail,
            creator: Some(creator),
        }
    }
}

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
            last_seen: None,
            episode_seen: HashMap::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub enum LoadingState {
    #[default]
    Loading,
    Completed,
    Error(String),
}

#[derive(Clone, Debug, Default, Store, Deserialize)]
pub struct UserData {
    #[allow(dead_code)]
    pub language: Language,
    pub webtoons: UserWebtoons,

    #[serde(default)]
    pub loading_state: LoadingState,
}

#[derive(Clone, Debug, Default, Store, Deserialize)]
pub struct UserRecommendations {
    pub webtoons: Vec<WebtoonSearchInfo>,

    #[serde(default)]
    pub loading_state: LoadingState,
}
