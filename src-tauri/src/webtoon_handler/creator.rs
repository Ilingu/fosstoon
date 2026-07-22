use tauri::{Emitter, Manager};
use webtoon_sdk::{
    creator::{fetch_creator_from_alias, WtCreator},
    DownloadState,
};

#[tauri::command(rename_all = "snake_case")]
pub async fn get_author_info(
    app: tauri::AppHandle,
    profile_id: String,
) -> Result<WtCreator, String> {
    let creator_dl_progress = |news: DownloadState| {
        let _ = app.emit("creator_dl_channel", news);
    };

    // episodes panels are stored temporarily in cache
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    fetch_creator_from_alias(&profile_id, &cache_dir, creator_dl_progress).await
}
