use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use regex::RegexSet;
use serde_json::Value;
use tokio::fs;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use kemono_api::model::posts_legacy::{PostsLegacy, Props, Result as PLResult};
use kemono_api::API;

use crate::utils::download_single;
use crate::DONE;

pub mod ctx;

pub async fn download_loop(ctx: impl ctx::Context<'_>) -> Result<()> {
    let web_name = ctx.web_name();
    let user_id = ctx.user_id();
    let max_concurrency = ctx.max_concurrency();
    let output_dir = ctx.output_dir();
    let whitelist_regex = ctx.whitelist_regexes();
    let blacklist_regex = ctx.blacklist_regexes();

    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let web_name = web_name.as_ref();
    let user_id = user_id.as_ref();
    let whitelist_regex = RegexSet::new(whitelist_regex)?;
    let blacklist_regex = RegexSet::new(blacklist_regex)?;

    let api = API::new();
    let mut offset = 0;

    loop {
        if DONE.load(Ordering::Relaxed) {
            break;
        }

        let PostsLegacy {
            props: Props { count, limit },
            results,
        } = api.get_posts_legacy(web_name, user_id, offset).await?;

        let clear_title_re = regex::Regex::new(r#"[\\/:*?"<>|]"#)?; // 替换特殊字串符

        for PLResult {
            id: ref post_id,
            ref title,
        } in results
        {
            if DONE.load(Ordering::Relaxed) {
                return Ok(());
            }

            if !whitelist_regex.is_empty() && !whitelist_regex.is_match(title) {
                info!("Skipped {title} due to whitelist mismatch");
                continue;
            }

            if blacklist_regex.is_match(title) {
                info!("Skipped {title} due to blacklist match");
                continue;
            }

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
