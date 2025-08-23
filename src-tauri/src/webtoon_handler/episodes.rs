use serde::{Deserialize, Serialize};
use webtoon::platform::webtoons::{self, webtoon::episode::AdStatus, Webtoon};

use crate::webtoon_handler::webtoon::WebtoonId;

pub type EpNum = u16;

#[derive(Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub wt_id: WebtoonId,

    /// the episode number
    pub number: EpNum,
    pub title: String,
    pub thumbnail: String,
    pub has_ad_wall: bool,
    pub views: Option<u32>,
    pub likes: u32,
    pub author_note: Option<String>,
    /// all the episode image/panel url
    pub panels_url: Vec<String>,
    // pub top_posts: Option<Vec<Post>>,
}

/// for the app simplicity sake, no replies will be fetch in this app
#[derive(Serialize, Deserialize)]
pub struct Post {
    pub wt_id: WebtoonId,
    pub ep_num: EpNum,

    pub id: String,
    pub content: String,
    pub is_spoiler: bool,
    pub upvotes: u32,
    pub downvotes: u32,
    pub posted_at: u64,
    pub poster_name: String,
}

impl EpisodeInfo {
    pub async fn fetch_posts(wt_id: WebtoonId, ep_num: u16) -> Result<Vec<Post>, String> {
        let wtclient = webtoons::Client::new();

        let webtoon = wtclient
            .webtoon(wt_id.wt_id, wt_id.wt_type)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("Webtoon not found")?;
        let episode = webtoon
            .episode(ep_num)
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

/* Helpers */

pub async fn fetch_episodes(webtoon: &Webtoon) -> Result<Vec<EpisodeInfo>, String> {
    let episodes = webtoon.episodes().await.map_err(|err| err.to_string())?;

    let mut episodes_info = vec![];
    for episode in episodes {
        let episode_info = EpisodeInfo {
            wt_id: WebtoonId::new(webtoon.id(), webtoon.r#type()),
            number: episode.number(),
            title: episode.title().await.map_err(|err| err.to_string())?,
            thumbnail: episode.thumbnail().await.map_err(|err| err.to_string())?,
            has_ad_wall: matches!(episode.ad_status(), Some(AdStatus::Yes)),
            views: episode.views(),
            likes: episode.likes().await.map_err(|err| err.to_string())?,
            author_note: episode.note().await.map_err(|err| err.to_string())?,
            panels_url: episode
                .panels()
                .await
                .map_err(|err| err.to_string())?
                .iter()
                .map(|p| p.url().to_string())
                .collect(),
        };
        episodes_info.push(episode_info);
    }

    Ok(episodes_info)
}

/* Commands */

#[tauri::command]
pub async fn get_episode_posts(id: WebtoonId, ep_num: u16) -> Result<Vec<Post>, String> {
    EpisodeInfo::fetch_posts(id, ep_num).await
}
