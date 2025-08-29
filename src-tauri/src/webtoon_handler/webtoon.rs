use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use nanorand::{Rng, WyRand};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use webtoon::platform::webtoons::{
    self, canvas, meta::Genre, originals::Schedule, Language, Type, Webtoon,
};
use webtoon_sdk::{episodes::EpisodePreview, WebtoonId as SDKWtId};

use crate::{constants::WEBTOONS_STORE, image_handler::download_images, store::UserData};

/* Type Definition */
#[derive(Serialize, Clone, Copy, Deserialize, Debug)]
pub struct WebtoonId {
    pub wt_id: u32,
    pub wt_type: Type,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub episodes: Option<Vec<EpisodePreview>>,
    pub refresh_eps_at: SystemTime,

    pub expired_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug)]
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

impl From<WebtoonId> for SDKWtId {
    fn from(value: WebtoonId) -> Self {
        Self {
            wt_id: value.wt_id,
            wt_type: match value.wt_type {
                Type::Original => webtoon_sdk::WtType::Original,
                Type::Canvas => webtoon_sdk::WtType::Canvas,
            },
        }
    }
}

impl From<SDKWtId> for WebtoonId {
    fn from(value: SDKWtId) -> Self {
        Self {
            wt_id: value.wt_id,
            wt_type: match value.wt_type {
                webtoon_sdk::WtType::Canvas => Type::Canvas,
                webtoon_sdk::WtType::Original => Type::Original,
            },
        }
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
    pub async fn fetch_episodes(&mut self, thumbnail_path: &Path) -> Result<(), String> {
        self.episodes = Some(webtoon_sdk::episodes::scrap_episodes_info(self.id.into()).await?);
        self.download_episodes_thumbnail(thumbnail_path).await?;

        Ok(())
    }

    /// **DOES NOT INCLUDE COMMENTS**
    pub async fn update_episodes(&mut self, thumbnail_path: &Path) -> Result<(), String> {
        if let Some(episodes) = self.episodes.as_mut() {
            let mut new_ep_since_last =
                webtoon_sdk::episodes::check_for_new_eps(self.id.into(), episodes.len()).await?;
            episodes.append(&mut new_ep_since_last);
            self.download_episodes_thumbnail(thumbnail_path).await?
        }

        Ok(())
    }

    /// locally downaload eps thumbnail and set the disk path as the new eps thumb url
    pub async fn download_episodes_thumbnail(
        &mut self,
        thumbnail_path: &Path,
    ) -> Result<(), String> {
        if let Some(eps) = self.episodes.as_mut() {
            let new_thumbnails_url = download_images(
                thumbnail_path,
                eps.iter().map(|e| e.thumbnail.clone()).collect(),
            )
            .await?;
            for (e, new_thumb_url) in eps.iter_mut().zip(new_thumbnails_url) {
                e.thumbnail = new_thumb_url
            }
        }
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
            refresh_eps_at: SystemTime::now()
                .checked_add(Duration::from_secs(86400)) // add 1 days before refresh
                .ok_or("are we near 2038?")?,

            expired_at: SystemTime::now()
                .checked_add(Duration::from_secs(864000)) // add 10 days before refresh
                .ok_or("are we near 2038?")?,
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
pub async fn get_webtoon_info(app: tauri::AppHandle, id: WebtoonId) -> Result<WebtoonInfo, String> {
    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;

    // check if already cached and not expired
    let webtoon_info = match webtoons_store
        .get(id.wt_id.to_string())
        .map(serde_json::from_value::<WebtoonInfo>)
    {
        Some(Ok(wt))
            if wt.expired_at < SystemTime::now() && wt.refresh_eps_at < SystemTime::now() =>
        {
            return Ok(wt); // no need to re-write the same value to the store
        }
        Some(Ok(mut wt)) if wt.expired_at >= SystemTime::now() => {
            // refresh expired webtoon
            wt.refresh().await?;
            wt
        }
        Some(Ok(mut wt)) if wt.refresh_eps_at >= SystemTime::now() => {
            // get missing eps
            wt.update_episodes(&app.path().app_local_data_dir().map_err(|e| e.to_string())?)
                .await?;
            wt
        }
        Some(Ok(mut wt)) => {
            // refresh expired webtoon
            wt.refresh().await?;
            // get missing eps
            wt.update_episodes(&app.path().app_local_data_dir().map_err(|e| e.to_string())?)
                .await?;
            wt
        }
        Some(Err(_)) | None => {
            // if not existing or type migration, fetch data
            let mut webtoon = WebtoonInfo::new_from_id(id).await?;
            webtoon
                .fetch_episodes(&app.path().app_local_data_dir().map_err(|e| e.to_string())?)
                .await?;
            webtoon
        }
    };

    // set updated/new webtoon to storage
    webtoons_store.set(
        id.wt_id.to_string(),
        serde_json::to_value(&webtoon_info).map_err(|_| "Couldn't serialize webtoon_info")?,
    );
    Ok(webtoon_info)
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
