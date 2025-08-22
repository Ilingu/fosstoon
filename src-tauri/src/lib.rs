mod webtoon_handler;

use webtoon::platform::webtoons::{errors::Error, Client, Type};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Haudi, {}! You've been greeted from the dev!", name)
}

#[tauri::command]
async fn get_image() -> Result<String, String> {
    let client = Client::new();

    let Some(webtoon) = client
        .webtoon(843910, Type::Canvas)
        .await
        .map_err(|_| "Failed to get webtoon".to_string())?
    else {
        panic!("No webtoon of given id and type exits");
    };

    let episode = webtoon
        .episode(1)
        .await
        .map_err(|_| "Failed to get ep".to_string())?
        .ok_or("No ep".to_string())?;

    let panels = episode
        .download()
        .await
        .map_err(|_| "Failed to save to disk".to_string())?;

    // Save as a single, long image.
    panels
        .save_single("examples/panels")
        .await
        .map_err(|_| "Failed to save to disk".to_string())?;

    Ok("".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // load user state

    tauri::Builder::default()
        // .manage(MyState("some state value".into())) -> inject user state into app
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
