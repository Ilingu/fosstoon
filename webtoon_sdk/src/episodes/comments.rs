use serde::{Deserialize, Serialize};

use crate::{
    WebtoonId,
    episodes::{
        EpisodeData,
        comments_resp_type::{CommentsRes, CommentsResp},
    },
};

/// for the app simplicity sake, no replies will be fetch in this app
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Post {
    pub wt_id: WebtoonId,
    pub ep_num: usize,

    pub id: String,
    pub content: String,
    pub is_spoiler: bool,
    pub is_top: bool,
    pub upvotes: u32,
    pub downvotes: u32,
    pub posted_at: u64,
    pub poster_name: String,
}

pub trait PostExtension {
    #[allow(async_fn_in_trait)]
    async fn fetch_posts(wt_id: WebtoonId, ep_num: usize) -> Result<Vec<Post>, String>;
}

impl PostExtension for EpisodeData {
    async fn fetch_posts(wt_id: WebtoonId, ep_num: usize) -> Result<Vec<Post>, String> {
        let http_client = reqwest::Client::new();

        let comments_url = {
            let wt_type_char = match wt_id.wt_type {
                crate::WtType::Canvas => 'c',
                crate::WtType::Original => 'w',
            };
            format!(
                "https://www.webtoons.com/p/api/community/v1/page/{}_{}_{}/posts/search?categoryId=&pinRepresentation=distinct&displayBlindCommentAsService=false&prevSize=0&nextSize=20",
                wt_type_char, wt_id.wt_id, ep_num
            )
        };

        let CommentsResp {
            result: CommentsRes { tops, posts },
        } = http_client
            .get(&comments_url)
            .header("service-ticket-id", "epicom")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<CommentsResp>()
            .await
            .map_err(|_| "Failed to deserialize CommentsResp")?;

        let posts = [tops, posts]
            .concat()
            .into_iter()
            .filter_map(|rc| TryInto::<Post>::try_into(rc).ok())
            .collect::<Vec<_>>();

        Ok(posts)
    }
}
