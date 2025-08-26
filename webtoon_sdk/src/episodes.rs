use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{WebtoonId, WtType};

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

pub async fn scrap_episodes_info(id: WebtoonId) -> Result<Vec<EpisodePreview>, String> {
    let ep_selector = Selector::parse("#_listUl > li").unwrap();
    let ep_url_selector = Selector::parse("a").unwrap();
    let date_selector = Selector::parse(".date").unwrap();
    let title_selector = Selector::parse(".subj > span").unwrap();
    let thumb_selector = Selector::parse(".thmb > img").unwrap();
    let likes_selector = Selector::parse(".like_area").unwrap();

    let mut episodes = vec![];
    let mut real_url = None;
    'outer: for page in 1.. {
        let url = match real_url {
            Some(ref rurl) => format!("{rurl}&page={page}"),
            None => match id.wt_type {
                WtType::Canvas => format!(
                    "https://www.webtoons.com/en/canvas/*/list?title_no={}&page={page}",
                    id.wt_id,
                ),
                WtType::Original => {
                    format!(
                        "https://www.webtoons.com/en/*/*/list?title_no={}&page={page}",
                        id.wt_id
                    )
                }
            },
        };

        let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
        if real_url.is_none() && page == 1 {
            real_url = Some(resp.url().to_string())
        }

        let raw_html = resp.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&raw_html);

        for element in document.select(&ep_selector) {
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

            episodes.push(EpisodePreview {
                parent_wt_id: id,
                number: ep_num,
                title,
                thumbnail,
                likes,
                posted_at: date,
                ep_url,
            });

            if ep_num == 1 {
                break 'outer;
            }
        }
    }

    episodes.reverse();
    Ok(episodes)
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
