use nanorand::{Rng, WyRand};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use webtoon::platform::webtoons::{
    self, canvas, meta::Genre, originals::Schedule, Language, Type, Webtoon,
};

use crate::{
    store::UserData,
    webtoon_handler::episodes::{fetch_episodes, EpisodeInfo},
};

/* Type Definition */
#[derive(Serialize, Clone, Copy, Deserialize)]
pub struct WebtoonId {
    pub wt_id: u32,
    pub wt_type: Type,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebtoonInfo {
    pub id: WebtoonId,

    pub title: String,
    pub thumbnail: Option<String>,
    pub language: Language,
    pub banner: Option<String>,
    pub creators: Vec<String>,
    pub genres: Vec<Genre>,
    pub schedule: Option<Schedule>,
    pub is_completed: bool,
    pub views: u64,
    pub likes: u32,
    pub subs: u32,
    pub summary: String,

    pub episodes: Option<Vec<EpisodeInfo>>,
}

#[derive(Serialize, Deserialize)]
pub struct WebtoonSearchInfo {
    id: WebtoonId,

    title: String,
    thumbnail: String,
    creator: String,
}

/* Implementations */

impl WebtoonId {
    pub fn new(id: u32, wt_type: Type) -> Self {
        Self { wt_id: id, wt_type }
    }
}

impl WebtoonInfo {
    /// gather all info for the requested webtoon
    ///
    /// **DOES NOT INCLUDE EPISODES** (for that you have to call the WebtoonInfo::fetch_episodes method)
    pub async fn new_from_id(id: WebtoonId) -> Result<Self, String> {
        let wtclient = webtoons::Client::new();

        let webtoon = wtclient
            .webtoon(id.wt_id, id.wt_type)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("Webtoon not found")?;

        Self::convert(&webtoon).await
    }

    /// **DOES NOT INCLUDE COMMENTS**
    pub async fn fetch_episodes(&mut self) -> Result<(), String> {
        self.update_episodes().await
    }

    /// **DOES NOT INCLUDE COMMENTS**
    pub async fn update_episodes(&mut self) -> Result<(), String> {
        let wtclient = webtoons::Client::new();

        let webtoon = wtclient
            .webtoon(self.id.wt_id, self.id.wt_type)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("Webtoon not found")?;

        self.episodes = Some(fetch_episodes(&webtoon).await?);
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<(), String> {
        *self = WebtoonInfo::new_from_id(self.id).await?;
        Ok(())
    }

    /// convert data from the private type Webtoon to the public type WebtoonInfo
    ///
    /// **DOES NOT INCLUDE EPISODES**
    pub async fn convert(webtoon: &Webtoon) -> Result<WebtoonInfo, String> {
        Ok(WebtoonInfo {
            id: WebtoonId::new(webtoon.id(), webtoon.r#type()),
            title: webtoon.title().await.map_err(|err| err.to_string())?,
            thumbnail: webtoon.thumbnail().await.map_err(|err| err.to_string())?,
            language: webtoon.language(),
            banner: webtoon.banner().await.map_err(|err| err.to_string())?,
            creators: webtoon
                .creators()
                .await
                .map_err(|err| err.to_string())?
                .iter()
                .map(|c| c.username().to_string())
                .collect(),
            genres: webtoon.genres().await.map_err(|err| err.to_string())?,
            schedule: webtoon.schedule().await.map_err(|err| err.to_string())?,
            is_completed: webtoon
                .is_completed()
                .await
                .map_err(|err| err.to_string())?,
            views: webtoon.views().await.map_err(|err| err.to_string())?,
            likes: webtoon.likes().await.map_err(|err| err.to_string())?,
            subs: webtoon.subscribers().await.map_err(|err| err.to_string())?,
            summary: webtoon.summary().await.map_err(|err| err.to_string())?,

            episodes: None,
        })
    }
}

/* Commands */

#[tauri::command]
pub async fn search_webtoon(
    user_state: tauri::State<'_, Mutex<UserData>>,
    query: &str,
) -> Result<Vec<WebtoonSearchInfo>, String> {
    let user_langage = user_state.lock().await.language;

    let client = webtoons::Client::new();
    let search_result = client
        .search(query, user_langage)
        .await
        .map_err(|err| err.to_string())?;

    let mut webtoon_found = vec![];
    for webtoon in search_result {
        webtoon_found.push(WebtoonSearchInfo {
            id: WebtoonId::new(webtoon.id(), webtoon.r#type()),

            title: webtoon.title().to_string(),
            thumbnail: webtoon.thumbnail().to_string(),
            creator: webtoon.creator().to_string(),
        });
    }

    Ok(webtoon_found)
}

#[tauri::command]
pub async fn get_webtoon_info(id: WebtoonId) -> Result<WebtoonInfo, String> {
    let mut webtoon = WebtoonInfo::new_from_id(id).await?;
    webtoon.fetch_episodes().await?;
    Ok(webtoon)
}

#[tauri::command]
/// get canvas and original (check exemple)
pub async fn get_homepage_recommandations(
    user_state: tauri::State<'_, Mutex<UserData>>,
) -> Result<Vec<WebtoonSearchInfo>, String> {
    let user_langage = user_state.lock().await.language;

    let wt_client = webtoons::Client::new();

    let originals = wt_client
        .originals(user_langage)
        .await
        .map_err(|err| err.to_string())?;

    let canvas = wt_client
        .canvas(user_langage, 1..=2, canvas::Sort::Popularity)
        .await
        .map_err(|err| err.to_string())?;

    let raw_recommandations = {
        let mut merged = [originals, canvas].concat();

        // shuffle vec
        let mut rng = WyRand::new();
        rng.shuffle(&mut merged);
        merged
    };

    let converted_recommandations =
        futures::future::join_all(raw_recommandations.iter().map(WebtoonInfo::convert))
            .await
            .into_iter()
            .collect::<Result<Vec<WebtoonInfo>, String>>()?;

    let final_recommandations = converted_recommandations
        .into_iter()
        .map(|webtoon| WebtoonSearchInfo {
            id: webtoon.id,
            title: webtoon.title,
            thumbnail: webtoon.thumbnail.unwrap_or_default(),
            creator: webtoon
                .creators
                .first()
                .map_or("No creator", |v| v)
                .to_string(),
        })
        .collect();
    Ok(final_recommandations)
}
