use serde::{Deserialize, Serialize};

use crate::creator::CreatorPost;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorPostResp {
    pub result: CreatorPostResult,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorPostResult {
    pub posts: Vec<RawCreatorPost>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawCreatorPost {
    pub body: String,
    pub section_group: SectionGroup,
    #[serde(default)]
    // pub child_post_previews: Vec<ChildPostPreview>,
    pub created_at: usize,
    // pub updated_at: usize,
    pub view_count: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionGroup {
    pub sections: Vec<Section>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub section_type: String,
    pub data: SectionData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionData {
    pub domain: Option<String>,
    pub path: Option<String>,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

/* To have child post preview, you must put a non null number into the corresponding api url param
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildPostPreview {
    pub body: String,
    pub created_by: CreatedBy,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedBy {
    pub name: String,
}
*/

impl From<RawCreatorPost> for CreatorPost {
    fn from(value: RawCreatorPost) -> Self {
        Self {
            body: value.body,
            created_at: value.created_at,
            img_url: value
                .section_group
                .sections
                .into_iter()
                .next()
                .and_then(|sec| {
                    (sec.section_type == "IMAGE"
                        && sec.data.domain.is_some()
                        && sec.data.path.is_some())
                    .then(|| {
                        format!(
                            "{}{}",
                            sec.data.domain.unwrap_or_default(),
                            sec.data.path.unwrap_or_default()
                        )
                    })
                }),
        }
    }
}
