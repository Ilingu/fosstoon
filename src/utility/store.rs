use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use reactive_stores::Store;

use crate::utility::types::{Language, WebtoonId, WebtoonSearchInfo};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserWebtoon {
    pub id: WebtoonId,
    pub title: String,
    pub thumbnail: Option<String>,
    pub creator: Option<String>,
    pub last_seen: Option<usize>,
    pub episode_seen: HashMap<usize, bool>,
}

pub type UserWebtoons = HashMap<usize, UserWebtoon>;

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
            thumbnail: thumbnail.unwrap_or_default(),
            creator,
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
