use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;
use webtoon::platform::webtoons::{self};
use webtoon_sdk::{episodes::EpisodeData, webtoon::WebtoonInfo, DownloadState, WebtoonId};

use crate::{constants::WEBTOONS_STORE, webtoon_handler::FromWtType};

/// for the app simplicity sake, no replies will be fetch in this app
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Post {
    pub wt_id: WebtoonId,
    pub ep_num: usize,

    pub id: String,
    pub content: String,
    pub is_spoiler: bool,
    pub upvotes: u32,
    pub downvotes: u32,
    pub posted_at: u64,
    pub poster_name: String,
}

pub trait PostExtension {
    async fn fetch_posts(wt_id: WebtoonId, ep_num: usize) -> Result<Vec<Post>, String>;
}

impl PostExtension for EpisodeData {
    async fn fetch_posts(wt_id: WebtoonId, ep_num: usize) -> Result<Vec<Post>, String> {
        let wtclient = webtoons::Client::new();

        let webtoon = wtclient
            .webtoon(wt_id.wt_id as u32, wt_id.wt_type.to_local_type())
            .await
            .map_err(|err| err.to_string())?
            .ok_or("Webtoon not found")?;
        let episode = webtoon
            .episode(ep_num as u16)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("Episode not found")?;

        let top_posts: Vec<Post> = episode
            .posts()
            .await
            .map_err(|err| err.to_string())?
            .filter(|p| p.is_top() && p.is_comment())
            .map(|p| Post {
                wt_id,
                ep_num,
                id: p.id().to_string(),
                content: p.body().contents().to_string(),
                is_spoiler: p.body().is_spoiler(),
                upvotes: p.upvotes(),
                downvotes: p.downvotes(),
                posted_at: p.posted() as u64,
                poster_name: p.poster().username().to_string(),
            })
            .collect();

        Ok(top_posts)
    }
}

/* Commands */

#[tauri::command(rename_all = "snake_case")]
pub async fn get_episode_post(wt_id: WebtoonId, ep_num: usize) -> Result<Vec<Post>, String> {
    webtoon_sdk::episodes::EpisodeData::fetch_posts(wt_id, ep_num).await
}

#[tauri::command]
pub async fn force_refresh_episodes(
    app: tauri::AppHandle,
    id: WebtoonId,
) -> Result<WebtoonInfo, String> {
    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;

    let mut updated_wt = match webtoons_store
        .get(id.wt_id.to_string())
        .map(serde_json::from_value::<WebtoonInfo>)
    {
        Some(Ok(mut wt)) => {
            let thumb_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
            wt.update_episodes(&thumb_path, |_| {}).await?;
            wt
        }
        Some(Err(_)) | None => return Err("webtoon not found".to_string()),
    };

    updated_wt.refresh_eps_at = SystemTime::now()
        .checked_add(Duration::from_secs(86400)) // add 1 days before refresh
        .ok_or("are we near 2038?")?;

    // set updated webtoon to storage
    webtoons_store.set(
        id.wt_id.to_string(),
        serde_json::to_value(&updated_wt).map_err(|_| "Couldn't serialize updated_wt")?,
    );
    Ok(updated_wt)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_episode_data(
    app: tauri::AppHandle,
    wt_id: WebtoonId,
    ep_num: usize,
) -> Result<(EpisodeData, bool), String> {
    if ep_num == 0 {
        return Err("episode number cannot be 0".to_string());
    }

    let dl_progress_cb = |news: DownloadState| {
        let _ = app.emit("ep_dl_channel", news);
    };

    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;

    let webtoon = webtoons_store
        .get(wt_id.wt_id.to_string())
        .map(serde_json::from_value::<WebtoonInfo>)
        .ok_or("No webtoon found in store")?
        .map_err(|e| e.to_string())?;
    let episodes = webtoon.episodes.ok_or("No episode found in store")?;

    let episode = episodes
        .get(ep_num - 1)
        .cloned()
        .ok_or("Requested episode not found in store")?;
    let has_next_ep = ep_num != episodes.len();

    let mut ep_data = episode.get_episode_data(dl_progress_cb).await?;

    // episodes panels are stored temporarily in cache
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    ep_data.dl_panels(&cache_dir, dl_progress_cb).await?;

    Ok((ep_data, has_next_ep))
}
