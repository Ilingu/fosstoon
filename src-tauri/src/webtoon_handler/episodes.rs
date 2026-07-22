use std::time::{Duration, SystemTime};

use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;
use webtoon_sdk::{
    episodes::{
        comments::{Post, PostExtension},
        EpisodeData, EpisodesExtraMethod,
    },
    webtoon::WebtoonInfo,
    DownloadState, WebtoonId,
};

use crate::constants::WEBTOONS_STORE;

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

    let has_next_ep = ep_num != episodes.get_last_ep_num();
    let episode = episodes
        .into_iter()
        .find(|ep| ep.number == ep_num)
        .ok_or("Requested episode not found in store")?;

    let mut ep_data = episode.get_episode_data(dl_progress_cb).await?;

    // episodes panels are stored temporarily in cache
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    ep_data.dl_panels(&cache_dir, dl_progress_cb).await?;

    Ok((ep_data, has_next_ep))
}
