use std::path::Path;

use futures::{stream::FuturesUnordered, StreamExt};
use tokio::fs;

use crate::WtDownloadingInfo;

// todo: reverif que ca marche
pub async fn download_images<F: Fn(WtDownloadingInfo) + Clone>(
    cache_dir: &Path,
    images_url: Vec<String>,
    info_cb: F,
) -> Result<Vec<String>, String> {
    info_cb(WtDownloadingInfo::CachingImages(0));

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
    let raw_images_data = {
        let futures = images_url_to_cache.iter().enumerate().map(|(i, iurl)| {
            let value = http_client.clone();
            async move {
                let resp = value
                    .get(iurl)
                    .header("Referer", "https://www.webtoons.com/")
                    .send()
                    .await?;

                Ok::<_, reqwest::Error>((i, resp.bytes().await))
            }
        });

        let requests_num = futures.len();
        let mut futures_unordered = FuturesUnordered::new();
        for f in futures {
            futures_unordered.push(f);
        }

        let mut responses_num = 0_usize;
        let mut responses_data = vec![None; requests_num];

        while let Some(result) = futures_unordered.next().await {
            let (order, bytes_resp) = result
                .map(|(i, b_resp)| match b_resp {
                    Ok(b) => Ok((i, b)),
                    Err(e) => Err(e.to_string()),
                })
                .map_err(|e| e.to_string())??;

            responses_num += 1;
            info_cb(WtDownloadingInfo::CachingImages(
                (((responses_num as f64) / (requests_num as f64)) * 100.0).round() as u8,
            ));
            responses_data[order] = Some(bytes_resp);
        }

        responses_data
            .into_iter()
            .map(|resp| resp.ok_or("Missing a response"))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    info_cb(WtDownloadingInfo::CachingImages(90));

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

    info_cb(WtDownloadingInfo::CachingImages(100));

    // return the path where the image are saved
    Ok(images_path)
}
