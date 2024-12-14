use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use kemono_api::API;
use serde_json::Value;
use tokio::fs;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use crate::utils::download_single;
use crate::DONE;

pub async fn download_loop(
    web_name: impl AsRef<str>,
    user_id: impl AsRef<str>,
    max_concurrency: usize,
    output_dir: &PathBuf,
) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let web_name = web_name.as_ref();
    let user_id = user_id.as_ref();

    let api = API::new();
    let mut offset = 0;

    loop {
        if DONE.load(Ordering::Relaxed) {
            break;
        }

        let posts_legacy_data = api.get_posts_legacy(web_name, user_id, offset).await?;
        let props = posts_legacy_data
            .get("props")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let limit = props.get("limit").and_then(Value::as_u64).unwrap_or(0) as usize;
        let count = props.get("count").and_then(Value::as_u64).unwrap_or(0) as usize;
        let results = posts_legacy_data
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let clear_title_re = regex::Regex::new(r#"[\\/:*?"<>|]"#)?; // 替换特殊字串符
        for result_val in results {
            if DONE.load(Ordering::Relaxed) {
                return Ok(());
            }

            let Some(post_id) = result_val.get("id").and_then(Value::as_str) else {
                error!("Post id not found");
                continue;
            };

            let Some(title) = result_val.get("title").and_then(Value::as_str) else {
                error!("Title Not Found");
                continue;
            };

            let post_data = api.get_post_info(&web_name, &user_id, post_id).await?;
            let attachments = post_data
                .get("attachments")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let previews = post_data
                .get("previews")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();

            let mut save_path = output_dir.join(title);
            if let Err(e) = fs::create_dir_all(&save_path).await {
                let es = format!("{}", e);
                if es.contains("267") || es.contains("Invalid argument") {
                    // 这就是 Windows .jpg
                    let new_title = clear_title_re.replace_all(title, "_");
                    save_path = output_dir.join(new_title.as_ref());
                    fs::create_dir_all(&save_path).await?;
                } else {
                    return Err(e.into());
                }
            }

            let mut futures = Vec::new();

            for attach in attachments.iter().chain(previews.iter()) {
                if DONE.load(Ordering::Relaxed) {
                    break;
                }

                let Some(file_name) = attach.get("name").and_then(Value::as_str) else {
                    warn!("File name not found in attachment");
                    continue;
                };
                let Some(file_server) = attach.get("server").and_then(Value::as_str) else {
                    warn!("File server not found");
                    continue;
                };
                let Some(file_path) = attach.get("path").and_then(Value::as_str) else {
                    warn!("File path not found");
                    continue;
                };

                let file_url = format!("{}/data{}", file_server, file_path);
                info!("Downloading file from {}", file_name);

                let api = api.clone();
                let sem = Arc::clone(&semaphore);
                let sp = save_path.clone();
                let fname = file_name.to_string();
                let furl = file_url.clone();
                let fut = async move {
                    let _permit = sem.acquire().await;
                    if let Err(e) = download_single(api, &furl, &sp, &fname).await {
                        error!("Error downloading {}: {:?}", fname, e);
                    }
                };
                futures.push(tokio::spawn(fut));
            }

            for f in futures {
                if let Err(e) = f.await {
                    error!("Task join error: {:?}", e);
                }
                if DONE.load(Ordering::Relaxed) {
                    return Ok(());
                }
            }

            if DONE.load(Ordering::Relaxed) {
                return Ok(());
            }
            info!("{} complete", title);
        }

        offset += limit;
        if offset > count {
            break;
        }
    }

    Ok(())
}
