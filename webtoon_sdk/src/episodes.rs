use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{generate_webtoon_url, WebtoonId, WtDownloadingInfo};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodePreview {
    pub parent_wt_id: WebtoonId,

    pub number: usize,
    pub title: String,
    pub thumbnail: String,
    pub likes: usize,
    pub posted_at: String,
    pub ep_url: String,
}

impl EpisodePreview {
    fn from_html_element(parent_id: WebtoonId, element: &ElementRef<'_>) -> Result<Self, String> {
        let ep_url_selector = Selector::parse("a").unwrap();
        let date_selector = Selector::parse(".date").unwrap();
        let title_selector = Selector::parse(".subj > span").unwrap();
        let thumb_selector = Selector::parse(".thmb > img").unwrap();
        let likes_selector = Selector::parse(".like_area").unwrap();

        let ep_num = element
            .attr("data-episode-no")
            .ok_or("No episode number found")?
            .parse::<usize>()
            .map_err(|e| e.to_string())?;

        let date = element
            .select(&date_selector)
            .next()
            .ok_or("No date")?
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        let title = element
            .select(&title_selector)
            .next()
            .ok_or("No title")?
            .text()
            .collect::<String>()
            .trim_end_matches("UP")
            .to_string();
        let thumbnail = element
            .select(&thumb_selector)
            .next()
            .ok_or("No thumbnail")?
            .attr("src")
            .ok_or("No ep thumb src")?
            .to_string();
        let likes = element
            .select(&likes_selector)
            .next()
            .ok_or("No likes")?
            .text()
            .collect::<String>()
            .trim_start_matches("like")
            .replace(",", "")
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        let ep_url = element
            .select(&ep_url_selector)
            .next()
            .ok_or("No ep url")?
            .attr("href")
            .ok_or("No ep url href")?
            .to_string();

        Ok(EpisodePreview {
            parent_wt_id: parent_id,
            number: ep_num,
            title,
            thumbnail,
            likes,
            posted_at: date,
            ep_url,
        })
    }
}

/* Functions */

pub enum ScrapEdgeCase {
    Inclusive,
    Exclusive,
}

async fn scrap_episodes_info_until<F: Fn(WtDownloadingInfo) + Clone>(
    id: WebtoonId,
    until_ep_id: usize,
    edge_case: ScrapEdgeCase,
    info_cb: F,
) -> Result<Vec<EpisodePreview>, String> {
    info_cb(WtDownloadingInfo::EpisodeInfo(0));

    let ep_selector = Selector::parse("#_listUl > li").unwrap();

    let mut progress = 0;

    let mut episodes = vec![];
    let mut real_url = None;
    'outer: for page in 1.. {
        let url = match real_url {
            Some(ref rurl) => format!("{rurl}&page={page}"),
            None => format!("{}&page={page}", generate_webtoon_url(id)),
        };

        let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
        if real_url.is_none() && page == 1 {
            real_url = Some(resp.url().to_string())
        }

        let raw_html = resp.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&raw_html);

        let mut last_ep_id = None;
        for element in document.select(&ep_selector) {
            let ep = EpisodePreview::from_html_element(id, &element)?;
            let ep_num = ep.number;

            match edge_case {
                ScrapEdgeCase::Inclusive => {
                    episodes.push(ep);
                    if ep_num <= until_ep_id {
                        break 'outer;
                    }
                }
                ScrapEdgeCase::Exclusive => {
                    if ep_num <= until_ep_id {
                        break 'outer;
                    }
                    episodes.push(ep);
                }
            }

            // anti-infinite loop
            if let Some(l) = last_ep_id
                && l == ep_num
            {
                break 'outer;
            }
            last_ep_id = Some(ep_num);
        }

        // update user feedback
        {
            progress = (progress + 10) % 100;
            info_cb(WtDownloadingInfo::EpisodeInfo(progress))
        }
    }

    info_cb(WtDownloadingInfo::EpisodeInfo(100));

    episodes.reverse();
    Ok(episodes)
}

/// checks for new released episodes in the webtoon described by its `id` since the last episode **number** stored in the
/// app storage
///
/// Therefore `last_stored_ep` start at `1` and not at `0` - becarful
///
/// It returns the potential missing episodes info (so if it returns an empty Vec there are no missing ep)
pub async fn check_for_new_eps<F: Fn(WtDownloadingInfo) + Clone>(
    id: WebtoonId,
    last_stored_ep: usize,
    info_cb: F,
) -> Result<Vec<EpisodePreview>, String> {
    scrap_episodes_info_until(id, last_stored_ep, ScrapEdgeCase::Exclusive, info_cb).await
}

pub async fn scrap_episodes_info<F: Fn(WtDownloadingInfo) + Clone>(
    id: WebtoonId,
    info_cb: F,
) -> Result<Vec<EpisodePreview>, String> {
    scrap_episodes_info_until(id, 1, ScrapEdgeCase::Inclusive, info_cb).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeData {
    pub parent_wt_id: WebtoonId,
    pub number: usize,

    pub panels: Vec<String>,
    pub author_note: String,
    pub author_name: String,
    pub author_thumb: String,
}

impl EpisodePreview {
    pub async fn get_episode_data(&self) -> Result<EpisodeData, String> {
        let raw_html = reqwest::get(&self.ep_url)
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let document = Html::parse_document(&raw_html);
        let panel_selector = Selector::parse("#_imageList > img").unwrap();
        let note_selector = Selector::parse(".author_text").unwrap();
        let name_selector = Selector::parse(".author_area .author_name").unwrap();
        let thumb_selector = Selector::parse(".author_area > .profile > img").unwrap();

        let mut panels = vec![];
        for img in document.select(&panel_selector) {
            panels.push(img.attr("data-url").ok_or("No panel url")?.to_string());
        }

        let author_note = document
            .select(&note_selector)
            .next()
            .ok_or("No note")?
            .text()
            .collect::<String>();
        let author_name = document
            .select(&name_selector)
            .next()
            .ok_or("No author name")?
            .text()
            .collect::<String>();
        let author_thumb = document
            .select(&thumb_selector)
            .next()
            .ok_or("No author thumb")?
            .attr("src")
            .ok_or("No author thumb src")?
            .to_string();

        Ok(EpisodeData {
            parent_wt_id: self.parent_wt_id,
            number: self.number,

            panels,
            author_note,
            author_name,
            author_thumb,
        })
    }
}
