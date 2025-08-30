use tauri::Manager;
use webtoon_sdk::image_dl::download_images;

#[tauri::command(rename_all = "snake_case")]
pub async fn fetch_wt_imgs(
    app: tauri::AppHandle,
    images_url: Vec<String>,
    temporary_cache: bool,
) -> Result<Vec<String>, String> {
    // get app cache dir
    let cache_dir = match temporary_cache {
        true => app.path().app_cache_dir(),
        false => app.path().app_local_data_dir(),
    }
    .map_err(|e| e.to_string())?;
    download_images(&cache_dir, images_url).await
}
