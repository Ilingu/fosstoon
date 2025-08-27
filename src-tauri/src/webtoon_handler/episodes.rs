use serde::{Deserialize, Serialize};
use tauri::Manager;
use webtoon::platform::webtoons::{self};
use webtoon_sdk::episodes::{EpisodeData, EpisodePreview};

use crate::{
    image_handler::{download_images, fetch_wt_imgs},
    webtoon_handler::webtoon::WebtoonId,
};

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
            .webtoon(wt_id.wt_id, wt_id.wt_type)
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

#[tauri::command]
pub async fn get_episode_post(id: WebtoonId, ep_num: usize) -> Result<Vec<Post>, String> {
    webtoon_sdk::episodes::EpisodeData::fetch_posts(id, ep_num).await
}

#[tauri::command]
pub async fn get_episode_data(
    app: tauri::AppHandle,
    ep: EpisodePreview,
) -> Result<EpisodeData, String> {
    let mut ep_data = ep.get_episode_data().await?;

    // episodes panels are stored temporarily in cache
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let local_path = download_images(cache_dir, ep_data.panels).await?;
    ep_data.panels = local_path;
    Ok(ep_data)
}
