use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use kemono_api::model::post_info::{AttachmentLike, PostInfo};
use kemono_api::model::user_profile::UserProfile;
use regex::RegexSet;
use tokio::fs;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, trace, warn};

use kemono_api::model::posts_legacy::{PostsLegacy, Props, Result as PLResult};
use kemono_api::API;

use crate::utils::{download_single, whiteblack_regex_filter};
use crate::DONE;

pub mod ctx;

pub async fn download_loop(ctx: impl ctx::Context<'_>) -> Result<()> {
    let web_name = ctx.web_name();
    let user_id = ctx.user_id();
    let max_concurrency = ctx.max_concurrency();
    let output_dir = ctx.output_dir();
    let whitelist_regex = ctx.whitelist_regexes();
    let blacklist_regex = ctx.blacklist_regexes();
    let whitelist_filename_regex = ctx.whitelist_filename_regexes();
    let blacklist_filename_regex = ctx.blacklist_filename_regexes();

    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let whitelist_regex = RegexSet::new(whitelist_regex)?;
    let blacklist_regex = RegexSet::new(blacklist_regex)?;
    let whitelist_filename_regex = RegexSet::new(whitelist_filename_regex)?;
    let blacklist_filename_regex = RegexSet::new(blacklist_filename_regex)?;

    let api = API::new();
    let mut offset = 0;

    // 替换特殊字串符
    let clear_title_re = regex::Regex::new(r#"[\\/:*?"<>|]"#)?;
    loop {
        if DONE.load(Ordering::Relaxed) {
            break;
        }

        let PostsLegacy {
            props: Props { count, limit },
            results,
        } = api
            .get_posts_legacy(web_name, user_id, offset)
            .await
            .map_err(|e| anyhow::anyhow!("failed to fetch props: {e}"))?;

        debug!("count: {count}, limit: {limit}");

        for PLResult {
            id: ref post_id,
            ref title,
        } in results
        {
            if DONE.load(Ordering::Relaxed) {
                return Ok(());
            }

            if !whiteblack_regex_filter(&whitelist_regex, &blacklist_regex, title) {
                info!("Skipped {title} by filter");
                continue;
            }

            let PostInfo {
                post,
                attachments,
                previews,
            } = api
                .get_post_info(web_name, user_id, post_id)
                .await
                .map_err(|e| anyhow::anyhow!("failed to get post info: {e}"))?;

            trace!("post: {post:?}");

            let UserProfile { ref public_id, .. } =
                api.get_user_profile(web_name, user_id)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to get user profile: {e}"))?;

            info!("user ({user_id}): {public_id}");

            let mut save_path = output_dir.join(public_id).join(title);
            if let Err(e) = fs::create_dir_all(&save_path).await {
                let es = format!("{}", e);
                if es.contains("267") || es.contains("Invalid argument") {
                    // 这就是 Windows .jpg
                    let new_title = clear_title_re.replace_all(title, "_");
                    save_path = output_dir.join(public_id).join(new_title.as_ref());
                    fs::create_dir_all(&save_path).await?;
                } else {
                    return Err(e.into());
                }
            }

            let metadata_path = save_path.join("metadata.json");
            fs::write(
                metadata_path,
                kemono_api::serde_json::to_string_pretty(&post)?,
            )
            .await?;

            let mut futures = Vec::new();

            info!("Downloading attachments from {title}");
            for attach in attachments.iter().chain(previews.iter()) {
                if DONE.load(Ordering::Relaxed) {
                    break;
                }

                let AttachmentLike {
                    server: Some(file_server),
                    name: Some(file_name),
                    path: Some(file_path),
                } = attach
                else {
                    warn!("missing field in attach {attach:?}");
                    continue;
                };

                if !whiteblack_regex_filter(
                    &whitelist_filename_regex,
                    &blacklist_filename_regex,
                    &file_name,
                ) {
                    info!("Skipped {file_name} by filter");
                    continue;
                }

                let file_url = format!("{}/data{}", file_server, file_path);
                info!("Downloading file from {}", file_name);

                let api = api.clone();
                let sem = Arc::clone(&semaphore);
                let sp = save_path.clone();
                let fname = file_name.to_string();
                let furl = file_url.clone();
                let fut = async move {
                    let _permit = sem.acquire().await;
                    match download_single(api, &furl, &sp, &fname).await {
                        Ok(()) => {
                            info!("Completed downloading {fname}");
                        }
                        Err(e) => {
                            error!("Error downloading {fname}: {e:?}");
                        }
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
