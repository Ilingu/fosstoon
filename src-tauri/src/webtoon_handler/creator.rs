use serde::{Deserialize, Serialize};

use crate::webtoon_handler::webtoon::WebtoonInfo;
use webtoon::platform::webtoons::{self, Language};

#[derive(Serialize, Deserialize)]
pub struct CreatorInfo {
    profile_id: String,

    name: String,

    followers: Option<u32>,
    webtoons: Vec<WebtoonInfo>,
}

impl CreatorInfo {
    pub async fn fetch_creator_data(
        profile_id: String,
        language: Language,
    ) -> Result<Self, String> {
        let wt_client = webtoons::Client::new();

        let creator = wt_client
            .creator(&profile_id, language)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("Creator not found")?;

        let creator_webtoons = futures::future::join_all(
            creator
                .webtoons()
                .await
                .map_err(|err| err.to_string())?
                .ok_or("Author has no webtoons")?
                .iter()
                .map(WebtoonInfo::convert),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<WebtoonInfo>, String>>()?;

        Ok(Self {
            profile_id: profile_id.to_string(),
            name: creator.username().to_string(),
            followers: creator.followers().await.map_err(|err| err.to_string())?,
            webtoons: creator_webtoons,
        })
    }
}

#[tauri::command]
pub async fn get_author_info(
    profile_id: String,
    language: Language,
) -> Result<CreatorInfo, String> {
    CreatorInfo::fetch_creator_data(profile_id, language).await
}
