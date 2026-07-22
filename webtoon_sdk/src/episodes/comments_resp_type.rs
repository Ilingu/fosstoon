use serde::{Deserialize, Serialize};

use crate::{WebtoonId, WtType, episodes::comments::Post};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentsResp {
    pub result: CommentsRes,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentsRes {
    pub tops: Vec<RawComment>,
    pub posts: Vec<RawComment>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawComment {
    pub page_id: String,
    pub id: String,

    pub body: String,
    pub created_by: CommentCreatedBy,
    pub created_at: u64,
    pub settings: CommentSettings,
    pub reactions: Vec<CommentReaction>,

    pub is_top: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreatedBy {
    pub id: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentReaction {
    pub reaction_id: String,
    pub emotions: Vec<CommentEmotion>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentEmotion {
    pub emotion_id: String,
    pub count: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSettings {
    /// "ON" OR "OFF"
    pub spoiler_filter: String,
}

impl TryFrom<RawComment> for Post {
    type Error = String;

    fn try_from(value: RawComment) -> Result<Self, Self::Error> {
        let (wt_type, wt_id, ep_num) = {
            let mut buf = value.page_id.split("_");
            let ctype = buf.next().ok_or("Comment ctype not found")?;
            let wt_id = buf
                .next()
                .ok_or("Comment wt_id not found")?
                .parse::<usize>()
                .map_err(|_| "failed to parse comment wt_id")?;
            let ep_num = buf
                .next()
                .ok_or("Comment ep_num not found")?
                .parse::<usize>()
                .map_err(|_| "failed to parse comment ep_num")?;
            (
                match ctype {
                    "w" => WtType::Original,
                    _ => WtType::Canvas,
                },
                wt_id,
                ep_num,
            )
        };

        let like_reaction = value
            .reactions
            .into_iter()
            .find(|r| r.reaction_id == "post_like")
            .unwrap_or_default();

        if value.created_by.name.is_empty() || value.body.is_empty() {
            return Err("Empty comment".to_string());
        }

        Ok(Self {
            wt_id: WebtoonId::new(wt_id, wt_type),
            ep_num,
            id: value.id,
            content: value.body,
            is_spoiler: value.settings.spoiler_filter == "ON",
            is_top: value.is_top,
            upvotes: like_reaction
                .emotions
                .iter()
                .find(|e| e.emotion_id == "like")
                .map(|e| e.count)
                .unwrap_or_default(),
            downvotes: like_reaction
                .emotions
                .iter()
                .find(|e| e.emotion_id == "dislike")
                .map(|e| e.count)
                .unwrap_or_default(),
            posted_at: value.created_at,
            poster_name: value.created_by.name,
        })
    }
}
