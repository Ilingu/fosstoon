use std::path::Path;

use futures::StreamExt;
use tokio::fs;

pub async fn download_images(
    cache_dir: &Path,
    images_url: Vec<String>,
) -> Result<Vec<String>, String> {
    // create images disk path
    let images_path = images_url
        .iter()
        .map(|url| {
            let filename = url
                .split("/")
                .last()
                .expect("Impossible no filename")
                .split("?")
                .next()
                .expect("Impossible no filename");
            cache_dir.join(filename).to_string_lossy().to_string()
        })
        .collect::<Vec<String>>();

    // check if some image are already cached
    let already_cached_images = futures::stream::iter(images_path.iter().enumerate())
        .filter_map(|(i, img_path)| async move {
            if fs::try_exists(img_path).await.unwrap_or_default() {
                Some(i)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .await;

    // remove already cached image from the url to fetch
    let images_url_to_cache = images_url
        .into_iter()
        .enumerate()
        .filter_map(|(i, url)| {
            if already_cached_images.contains(&i) {
                None
            } else {
                Some(url)
            }
        })
        .collect::<Vec<String>>();

    // fetch image data
    let http_client = reqwest::Client::new();
    let images_resp = {
        let futures = images_url_to_cache.iter().map(|iurl| {
            http_client
                .get(iurl)
                .header("Referer", "https://www.webtoons.com/")
                .send()
        });
        futures::future::join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    let raw_images_data =
        futures::future::join_all(images_resp.into_iter().map(|resp| resp.bytes()))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

    // write to disk
    futures::future::join_all(
        images_path
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if already_cached_images.contains(&i) {
                    None
                } else {
                    Some(p)
                }
            })
            .zip(raw_images_data)
            .map(|(dest, img_data)| fs::write(dest, img_data)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    // return the path where the image are saved
    Ok(images_path)
}
