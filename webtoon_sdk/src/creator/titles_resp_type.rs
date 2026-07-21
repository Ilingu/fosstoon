use serde::{Deserialize, Serialize};

use crate::{WebtoonId, WtType, search::WebtoonSearchInfo};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorWtResp {
    pub result: CreatorWtResult,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CreatorWtResult {
    pub titles: Vec<WtTitleInfo>,
    // pub total_count: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct WtTitleInfo {
    pub id: String,
    pub grade: String,
    #[serde(rename = "subject")]
    pub wt_name: String,
    pub thumbnail_url: String,
    pub authors: Vec<TitleAuthor>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleAuthor {
    pub nickname: String,
}

impl From<WtTitleInfo> for WebtoonSearchInfo {
    fn from(value: WtTitleInfo) -> Self {
        let webtoon_type = match value.grade.to_uppercase().as_ref() {
            "WEBTOON" => WtType::Original,
            _ => WtType::Canvas,
        };

        Self {
            id: WebtoonId::new(value.id.parse().unwrap(), webtoon_type),
            title: value.wt_name,
            thumbnail: value.thumbnail_url,
            creator: value.authors.into_iter().next().map(|a| a.nickname),
        }
    }
}
