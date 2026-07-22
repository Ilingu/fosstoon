mod posts_resp_type;
mod titles_resp_type;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    CreatorDownloadState, DownloadState,
    creator::{
        posts_resp_type::{CreatorPostResp, CreatorPostResult},
        titles_resp_type::{CreatorWtResp, CreatorWtResult},
    },
    image_dl::download_images,
    search::WebtoonSearchInfo,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatorPost {
    pub body: String,
    pub created_at: usize,
    pub img_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WtCreator {
    /// author's alias ID
    pub aid: String,
    /// true author ID
    pub taid: String,
    pub name: String,
    pub bio: Option<String>,
    pub followers: usize,
    pub webtoons: Vec<WebtoonSearchInfo>,
    pub posts: Vec<CreatorPost>,
}

impl WtCreator {
    /// only parse id, name, desc, followers count
    fn basic_info_from_html_page(raw_html: &str) -> Result<Self, String> {
        let author_true_id = raw_html
            .split_once(r#"\"creatorId\":\""#)
            .ok_or("Cannot find author encoded id")?
            .1
            .split_once(r#"\""#)
            .ok_or("Cannot find author true id end")?
            .0;

        let name = raw_html
            .split_once(r#"\"nickname\":\""#)
            .ok_or("Cannot find author nickname")?
            .1
            .split_once(r#"\""#)
            .ok_or("Cannot find author nickname end")?
            .0;

        let followers = {
            let temp = raw_html
                .split_once(r#"\"followerCount\":"#)
                .ok_or("Cannot find author followerCount")?
                .1;
            match temp.split_once(r#",\""#) {
                Some((v, _)) => v,
                None => {
                    temp.split_once("}")
                        .ok_or("Cannot find author followerCount end")?
                        .0
                }
            }
            .parse::<usize>()
            .map_err(|_| "Couldn't parse creator follower count")?
        };

        let bio = raw_html
            .split_once(r#"\"bio\":\""#)
            .and_then(|(_, v)| v.split_once(r#"\""#).map(|(v, _)| v.to_string()));

        Ok(Self {
            aid: "".to_string(),
            taid: author_true_id.to_string(),
            name: name.to_string(),
            bio,
            followers,
            webtoons: vec![],
            posts: vec![],
        })
    }
}

pub async fn fetch_creator_from_alias<F: Fn(DownloadState) + Clone>(
    author_id_alias: &str,
    app_cache_path: &Path,
    info_cb: F,
) -> Result<WtCreator, String> {
    //  fetch creator's basic info
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Basic(0)));
    let basic_info_url = format!("https://www.webtoons.com/p/community/en/u/{author_id_alias}");
    let basic_info_resp = reqwest::get(&basic_info_url)
        .await
        .map_err(|e| e.to_string())?;
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Basic(40)));

    let raw_html = basic_info_resp.text().await.map_err(|e| e.to_string())?;
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Basic(50)));

    let mut creator = WtCreator::basic_info_from_html_page(&raw_html)?;
    creator.aid = author_id_alias.to_string();
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Basic(100)));

    // Fetch creator's titles
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Title(0)));

    let creator_webtoons_url = format!(
        "https://www.webtoons.com/p/community/api/v1/creator/{}/titles?language=ENGLISH&nextSize=50",
        creator.taid
    );
    let CreatorWtResp {
        result: CreatorWtResult {
            titles: raw_resp_webtoons,
        },
    } = reqwest::get(&creator_webtoons_url)
        .await
        .map_err(|e| e.to_string())?
        .json::<CreatorWtResp>()
        .await
        .map_err(|_| "Failed to deserialize CreatorWtResp")?;
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Title(50)));

    let mut webtoons = raw_resp_webtoons
        .into_iter()
        .map(Into::<WebtoonSearchInfo>::into)
        .collect::<Vec<_>>();
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Title(100)));

    let new_wt_thumb_path = download_images(
        app_cache_path,
        webtoons.iter().map(|w| w.thumbnail.clone()).collect(),
        "creator_wt_thumbnail".to_string(),
        info_cb.clone(),
    )
    .await?
    .into_iter(); // download thumbnail img
    for (wt, new_path) in webtoons.iter_mut().zip(new_wt_thumb_path) {
        wt.thumbnail = new_path
    } // merge back with the right path to the downloaded img

    creator.webtoons = webtoons;

    // Fetch creator's posts & download post img
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Posts(0)));
    let creator_posts_url = format!(
        "https://www.webtoons.com/p/community/api/v2/posts?pageId={}&nextSize=10&cursor=&childPreviewCount=0&pinRepresentation=distinct",
        creator.taid
    );
    let CreatorPostResp {
        result: CreatorPostResult { posts: raw_posts },
    } = reqwest::get(&creator_posts_url)
        .await
        .map_err(|e| e.to_string())?
        .json::<CreatorPostResp>()
        .await
        .map_err(|_| "Failed to deserialize CreatorPostResp")?;
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Posts(50)));

    let mut posts = raw_posts
        .into_iter()
        .map(Into::<CreatorPost>::into)
        .collect::<Vec<_>>();
    info_cb(DownloadState::CreatorData(CreatorDownloadState::Posts(100)));

    let mut new_postsimg_path = download_images(
        app_cache_path,
        posts.iter().filter_map(|p| p.img_url.clone()).collect(),
        "creator_post_img".to_string(),
        info_cb.clone(),
    )
    .await?
    .into_iter(); // download posts img, if any
    for post in posts.iter_mut() {
        if let Some(img_path) = &mut post.img_url {
            *img_path = new_postsimg_path
                .next()
                .ok_or("Mismatch between original and final number of posts img")?;
        }
    } // merge back with the right path to the downloaded img

    creator.posts = posts;

    // Finished: sending back the data
    info_cb(DownloadState::Completed);
    Ok(creator)
}
