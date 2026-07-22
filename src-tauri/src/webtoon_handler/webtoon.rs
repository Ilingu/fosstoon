use std::time::SystemTime;

use nanorand::{Rng, WyRand};
use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;
use webtoon_sdk::{
    image_dl::download_images,
    recommandations::{fetch_canvas, fetch_original},
    search::WebtoonSearchInfo,
    webtoon::WebtoonInfo,
    DownloadState, WebtoonId,
};

use crate::constants::WEBTOONS_STORE;
/* Implementations */

/* Commands */

#[tauri::command]
pub async fn search_webtoon(
    app: tauri::AppHandle,
    query: &str,
) -> Result<Vec<WebtoonSearchInfo>, String> {
    let mut search_result = WebtoonSearchInfo::from_query(query).await?;

    let cache_thumb_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let new_thumb_path = download_images(
        &cache_thumb_path,
        search_result
            .iter()
            .map(|wt| wt.thumbnail.clone())
            .collect(),
        "search_webtoon".to_string(),
        |_| {},
    )
    .await?;
    for (wt, new_path) in search_result.iter_mut().zip(new_thumb_path) {
        wt.thumbnail = new_path
    }

    Ok(search_result)
}

#[tauri::command]
pub async fn delete_episodes(
    app: tauri::AppHandle,
    id: WebtoonId,
    eps2delete: Vec<usize>,
) -> Result<WebtoonInfo, String> {
    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;

    let mut webtoon = webtoons_store
        .get(id.wt_id.to_string())
        .map(serde_json::from_value::<WebtoonInfo>)
        .ok_or("Webtoon is not found")?
        .map_err(|e| e.to_string())?;

    let episodes = webtoon.episodes.ok_or("No episodes found")?;
    let filtered_eps = episodes
        .into_iter()
        .filter(|ep| !eps2delete.contains(&ep.number))
        .collect::<Vec<_>>();

    webtoon.episodes = Some(filtered_eps);

    // set updated/new webtoon to storage
    webtoons_store.set(
        id.wt_id.to_string(),
        serde_json::to_value(&webtoon).map_err(|_| "Couldn't serialize webtoon_info")?,
    );
    Ok(webtoon)
}

#[tauri::command]
pub async fn delete_webtoon(app: tauri::AppHandle, id: WebtoonId) -> Result<(), String> {
    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;

    // TODO: should also delete images
    match webtoons_store.delete(id.wt_id.to_string()) {
        true => Ok(()),
        false => Err("Failed to delete webtoon from the store".to_string()),
    }
}

#[tauri::command]
pub async fn get_webtoon_info(app: tauri::AppHandle, id: WebtoonId) -> Result<WebtoonInfo, String> {
    let wt_dl_progress_cb = |news: DownloadState| {
        let _ = app.emit("wt_dl_channel", news);
    };

    let webtoons_store = app
        .store(WEBTOONS_STORE)
        .map_err(|_| "Failed to open wt store")?;

    // check if already cached and not expired
    let webtoon_info = match webtoons_store
        .get(id.wt_id.to_string())
        .map(serde_json::from_value::<WebtoonInfo>)
    {
        Some(Ok(wt))
            if wt.expired_at > SystemTime::now() && wt.refresh_eps_at > SystemTime::now() =>
        {
            return Ok(wt); // no need to re-write the same value to the store
        }
        Some(Ok(mut wt))
            if wt.expired_at <= SystemTime::now() && wt.refresh_eps_at > SystemTime::now() =>
        {
            // refresh expired webtoon
            let thumb_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
            wt.refresh(&thumb_path, wt_dl_progress_cb).await?;
            wt
        }
        Some(Ok(mut wt))
            if wt.refresh_eps_at <= SystemTime::now() && wt.expired_at > SystemTime::now() =>
        {
            // get missing eps
            let thumb_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
            wt.update_episodes(&thumb_path, wt_dl_progress_cb).await?;
            wt
        }
        Some(Ok(mut wt)) => {
            // refresh expired webtoon
            let thumb_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
            wt.refresh(&thumb_path, wt_dl_progress_cb).await?;

            // get missing eps
            wt.update_episodes(&thumb_path, wt_dl_progress_cb).await?;
            wt
        }
        Some(Err(_)) | None => {
            // if not existing or type migration, fetch data
            let thumb_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
            let mut webtoon = WebtoonInfo::new_from_id(id, wt_dl_progress_cb).await?;
            webtoon
                .dl_wt_thumbnail(&thumb_path, wt_dl_progress_cb)
                .await?;
            webtoon
                .fetch_episodes(&thumb_path, wt_dl_progress_cb)
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
    app: tauri::AppHandle,
) -> Result<Vec<WebtoonSearchInfo>, String> {
    let mut canvas = fetch_canvas().await?;
    let original = fetch_original().await?;

    let mut merged = original;
    merged.append(&mut canvas);

    let mut rng = WyRand::new();
    rng.shuffle(&mut merged);

    let cache_thumb_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let new_thumb_path = download_images(
        &cache_thumb_path,
        merged.iter().map(|wt| wt.thumbnail.clone()).collect(),
        "homepage_recommandations".to_string(),
        |_| {},
    )
    .await?;
    for (wt, new_path) in merged.iter_mut().zip(new_thumb_path) {
        wt.thumbnail = new_path
    }

    Ok(merged)
}
