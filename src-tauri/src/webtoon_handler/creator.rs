use tauri::Manager;
use webtoon_sdk::creator::{fetch_creator_from_alias, WtCreator};

#[tauri::command(rename_all = "snake_case")]
pub async fn get_author_info(
    app: tauri::AppHandle,
    profile_id: String,
) -> Result<WtCreator, String> {
    // episodes panels are stored temporarily in cache
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    fetch_creator_from_alias(&profile_id, &cache_dir).await
}
