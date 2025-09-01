use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{WebtoonId, WtType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebtoonSearchInfo {
    pub id: WebtoonId,
    pub title: String,
    /// url
    pub thumbnail: String,

    /// option because depending on whether it's an orignal or not the data can't be scrapped easily
    pub creator: Option<String>,
}

impl WebtoonSearchInfo {
    pub async fn from_query(query: &str) -> Result<Vec<WebtoonSearchInfo>, String> {
        let encoded_query = urlencoding::encode(query);
        let resp = reqwest::get(format!(
            "https://www.webtoons.com/en/search?keyword={encoded_query}"
        ))
        .await
        .map_err(|e| e.to_string())?;

        let raw_html = resp.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&raw_html);

        let webtoons_selectors = Selector::parse(".webtoon_list > li > a").unwrap();
        let thumb_selector = Selector::parse(".image_wrap > img").unwrap();
        let title_selector = Selector::parse(".info_text > .title").unwrap();
        let author_selector = Selector::parse(".info_text > .author").unwrap();

        let mut search_results = vec![];
        for wt_elem in document.select(&webtoons_selectors) {
            let wt_id = wt_elem
                .attr("data-title-no")
                .ok_or("No wt id")?
                .trim()
                .parse::<usize>()
                .map_err(|e| e.to_string())?;
            let wt_type = match wt_elem
                .attr("data-webtoon-type")
                .ok_or("No wt type")?
                .to_lowercase()
                .trim()
            {
                "webtoon" => WtType::Original,
                "challenge" => WtType::Canvas,
                _ => return Err("Failed to parse wt type".to_string()),
            };

            let title = wt_elem
                .select(&title_selector)
                .next()
                .ok_or("No title".to_string())?
                .text()
                .collect::<String>()
                .trim()
                .to_string();
            let thumbnail = wt_elem
                .select(&thumb_selector)
                .next()
                .ok_or("No thumbnail".to_string())?
                .attr("src")
                .ok_or("No src".to_string())?
                .to_string();
            let creator = wt_elem
                .select(&author_selector)
                .next()
                .ok_or("No author".to_string())?
                .text()
                .collect::<String>()
                .trim()
                .to_string();

            search_results.push(WebtoonSearchInfo {
                id: WebtoonId::new(wt_id, wt_type),
                title,
                thumbnail,
                creator: Some(creator),
            });
        }

        Ok(search_results)
    }
}
