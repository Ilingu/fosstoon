use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    episodes::{check_for_new_eps, scrap_episodes_info, EpisodePreview},
    generate_webtoon_url,
    image_dl::download_images,
    DownloadState, Genre, Schedule, WebtoonId, WtType,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebtoonInfo {
    pub id: WebtoonId,

    pub title: String,
    pub thumbnail: String,
    pub banner: Option<String>,
    pub creators: Vec<String>,
    pub creator_id: Option<String>,
    pub genres: Vec<Genre>,
    pub schedule: Option<Schedule>,
    pub views: String,
    pub subs: String,
    pub summary: String,

    pub episodes: Option<Vec<EpisodePreview>>,
    pub refresh_eps_at: SystemTime,

    pub expired_at: SystemTime,
}

impl WebtoonInfo {
    /// gather all info for the requested webtoon
    ///
    /// **DOES NOT INCLUDE EPISODES** (for that you have to call the WebtoonInfo::fetch_episodes method)
    pub async fn new_from_id<F: Fn(DownloadState) + Clone>(
        id: WebtoonId,
        info_cb: F,
    ) -> Result<Self, String> {
        info_cb(DownloadState::WebtoonData(10));

        let title_selector = Selector::parse(".detail_header .subj").unwrap();
        let thumb_selector = Selector::parse(".detail_header > .thmb > img").unwrap();
        let banner_selector = Selector::parse("#content > .detail_bg").unwrap();
        let creators_selector = Selector::parse(".detail_header .author_area").unwrap();
        let creator_id_selector = Selector::parse(".detail_header .author_area > a").unwrap();
        let genre_selector = Selector::parse(".detail_header .genre").unwrap();
        let schedule_selector = Selector::parse(".detail_body .day_info").unwrap();
        let grade_selector = Selector::parse(".detail_body .grade_area .cnt").unwrap();
        let summary_selector = Selector::parse(".detail_body .summary").unwrap();

        let url = generate_webtoon_url(id);
        let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;

        info_cb(DownloadState::WebtoonData(50));

        let raw_html = resp.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&raw_html);

        info_cb(DownloadState::WebtoonData(80));

        let title = document
            .select(&title_selector)
            .next()
            .ok_or("No title")?
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        let thumbnail = document
            .select(&thumb_selector)
            .next()
            .ok_or("No thumb")?
            .attr("src")
            .ok_or("No src")?
            .to_string();
        let banner = match id.wt_type {
            crate::WtType::Canvas => None,
            crate::WtType::Original => Some(
                document
                    .select(&banner_selector)
                    .next()
                    .ok_or("No banner")?
                    .attr("style")
                    .ok_or("No style")?
                    .trim_start_matches("background:url('")
                    .trim_end_matches("') repeat-x")
                    .to_string(),
            ),
        };
        let creators = {
            let thumb_elem = document
                .select(&creators_selector)
                .next()
                .ok_or("No creators")?;

            if let Some(a) = thumb_elem.select(&Selector::parse("a").unwrap()).next() {
                vec![a.text().collect::<String>().trim().to_string()]
            } else {
                thumb_elem
                    .text()
                    .collect::<String>()
                    .trim()
                    .split(", ")
                    .map(|a| a.trim().replace("author info", "").trim().to_string())
                    .collect::<Vec<String>>()
            }
        };
        let creator_id = match creators.len() == 1 {
            true => Some(
                document
                    .select(&creator_id_selector)
                    .next()
                    .ok_or("No creator id")?
                    .attr("href")
                    .ok_or("No href aid")?
                    .split("/")
                    .last()
                    .map(|aid| aid.to_string())
                    .ok_or("Author Id not found".to_string())?,
            ),
            false => None,
        };
        let schedule = match id.wt_type {
            crate::WtType::Canvas => None,
            crate::WtType::Original => {
                let raw_schedule = document
                    .select(&schedule_selector)
                    .next()
                    .ok_or("No schedule")?
                    .text()
                    .collect::<String>()
                    .trim()
                    .trim_start_matches("UP")
                    .to_string();

                Some(raw_schedule.try_into()?)
            }
        };
        let genres = document
            .select(&genre_selector)
            .map(|g| g.text().collect::<String>().into())
            .collect::<Vec<Genre>>();

        let (views, subs) = match document
            .select(&grade_selector)
            .map(|gr| gr.text().collect::<String>())
            .collect::<Vec<_>>()
            .as_slice()
        {
            [views, subs, ..] => (views.to_owned(), subs.to_owned()),
            _ => return Err("wrong format for webtoon views and subs".to_string()),
        };
        let summary = document
            .select(&summary_selector)
            .next()
            .ok_or("No summary")?
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        info_cb(DownloadState::WebtoonData(100));

        Ok(Self {
            id,
            title,
            thumbnail,
            banner,
            creators,
            creator_id,
            genres,
            schedule,
            views,
            subs,
            summary,
            episodes: None,
            refresh_eps_at: SystemTime::now()
                .checked_add(Duration::from_secs(86400)) // add 1 days before refresh
                .ok_or("are we near 2038?")?,

            expired_at: SystemTime::now()
                .checked_add(Duration::from_secs(864000)) // add 10 days before refresh
                .ok_or("are we near 2038?")?,
        })
    }

    pub async fn dl_wt_thumbnail<F: Fn(DownloadState) + Clone>(
        &mut self,
        thumbnail_path: &Path,
        info_cb: F,
    ) -> Result<(), String> {
        if let WtType::Original = self.id.wt_type
            && fs::try_exists(thumbnail_path.join(format!("Thumb_Poster_{}.jpg", self.id.wt_id)))
                .await
                .unwrap_or_default()
        {
            self.thumbnail = thumbnail_path
                .join(format!("Thumb_Poster_{}.jpg", self.id.wt_id))
                .to_string_lossy()
                .to_string();
            return Ok(());
        }

        let new_thumb_path = {
            let dl_thumb_path = download_images(
                thumbnail_path,
                vec![self.thumbnail.clone()],
                info_cb.clone(),
            )
            .await?;
            match dl_thumb_path.as_slice() {
                [first, ..] => first.to_owned(),
                _ => return Err("expected one thumb download, found multiple".to_string()),
            }
        };
        self.thumbnail = new_thumb_path;
        Ok(())
    }

    /// **DOES NOT INCLUDE COMMENTS**
    pub async fn fetch_episodes<F: Fn(DownloadState) + Clone>(
        &mut self,
        thumbnail_path: &Path,
        info_cb: F,
    ) -> Result<(), String> {
        self.episodes = Some(scrap_episodes_info(self.id, info_cb.clone()).await?);
        self.download_episodes_thumbnail(thumbnail_path, info_cb)
            .await?;

        Ok(())
    }

    /// **DOES NOT INCLUDE COMMENTS**
    pub async fn update_episodes<F: Fn(DownloadState) + Clone>(
        &mut self,
        thumbnail_path: &Path,
        info_cb: F,
    ) -> Result<(), String> {
        if let Some(episodes) = self.episodes.as_mut() {
            let mut new_ep_since_last =
                check_for_new_eps(self.id, episodes.len(), info_cb.clone()).await?;
            episodes.append(&mut new_ep_since_last);
            self.download_episodes_thumbnail(thumbnail_path, info_cb)
                .await?
        } else {
            self.fetch_episodes(thumbnail_path, info_cb).await?
        }

        Ok(())
    }

    /// locally downaload eps thumbnail and set the disk path as the new eps thumb url
    pub async fn download_episodes_thumbnail<F: Fn(DownloadState) + Clone>(
        &mut self,
        thumbnail_path: &Path,
        info_cb: F,
    ) -> Result<(), String> {
        if let Some(eps) = self.episodes.as_mut() {
            let new_thumbnails_url = download_images(
                thumbnail_path,
                eps.iter().map(|e| e.thumbnail.clone()).collect(),
                info_cb,
            )
            .await?;
            for (e, new_thumb_url) in eps.iter_mut().zip(new_thumbnails_url) {
                e.thumbnail = new_thumb_url
            }
        }
        Ok(())
    }

    pub async fn refresh<F: Fn(DownloadState) + Clone>(
        &mut self,
        thumbnail_path: &Path,
        info_cb: F,
    ) -> Result<(), String> {
        let eps = self.episodes.clone();
        *self = WebtoonInfo::new_from_id(self.id, info_cb.clone()).await?;
        self.episodes = eps;

        self.dl_wt_thumbnail(thumbnail_path, info_cb).await?;
        Ok(())
    }
}
