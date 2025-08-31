use nanorand::{Rng, WyRand};
use scraper::{Html, Selector};

use crate::{webtoon::WebtoonSearchInfo, WebtoonId, WtType};

pub async fn fetch_original() -> Result<Vec<WebtoonSearchInfo>, String> {
    let resp = reqwest::get("https://www.webtoons.com/en/originals")
        .await
        .map_err(|e| e.to_string())?;

    let raw_html = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&raw_html);

    let webtoons_selectors = Selector::parse(".webtoon_list > li").unwrap();
    let id_selector = Selector::parse("a").unwrap();
    let title_selector = Selector::parse(".title").unwrap();
    let thumb_selector = Selector::parse(".image_wrap > img").unwrap();

    let mut todays_originals = vec![];
    for wt_elem in document.select(&webtoons_selectors) {
        let id = wt_elem
            .select(&id_selector)
            .next()
            .ok_or("No id :(".to_string())?
            .attr("data-title-no")
            .ok_or("No id")?
            .trim()
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
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

        todays_originals.push(WebtoonSearchInfo {
            id: WebtoonId::new(id, WtType::Original),
            title,
            thumbnail,
            creator: None,
        });

        if todays_originals.len() >= 20 {
            break;
        }
    }

    Ok(todays_originals)
}

pub async fn fetch_canvas() -> Result<Vec<WebtoonSearchInfo>, String> {
    let canvas_page = {
        let mut rng = WyRand::new();
        rng.generate_range(1_u8..=5)
    };

    let resp = reqwest::get(&format!(
        "https://www.webtoons.com/en/canvas/list?genreTab=ALL&sortOrder=MANA&page={canvas_page}",
    ))
    .await
    .map_err(|e| e.to_string())?;

    let raw_html = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&raw_html);

    let webtoons_selectors = Selector::parse(".challenge_lst li").unwrap();
    let id_selector = Selector::parse("a").unwrap();
    let title_selector = Selector::parse(".subj").unwrap();
    let thumb_selector = Selector::parse(".img_area > img").unwrap();
    let author_selector = Selector::parse(".author").unwrap();

    let mut canvas = vec![];
    for wt_elem in document.select(&webtoons_selectors) {
        let id = wt_elem
            .select(&id_selector)
            .next()
            .ok_or("No id :(".to_string())?
            .attr("href")
            .ok_or("No id")?
            .trim()
            .split("=")
            .nth(1)
            .ok_or("No id")?
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
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

        canvas.push(WebtoonSearchInfo {
            id: WebtoonId::new(id, WtType::Canvas),
            title,
            thumbnail,
            creator: Some(creator),
        });
    }

    Ok(canvas)
}
