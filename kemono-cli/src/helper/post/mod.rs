use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use anyhow::{anyhow, Result};
use kemono_api::model::post_info::{AttachmentLike, Post, PostInfo};
use regex::RegexSet;
use tokio::fs;
use tokio::task::JoinSet;
use tracing::{debug, error, info, trace, warn};

use kemono_api::API;

use crate::helper::ctx;
use crate::utils::{normalize_pathname, whiteblack_regex_filter};
use crate::DONE;

mod model;
use model::Attachment;
mod worker;
use worker::{worker, Payload};

#[tracing::instrument(skip(ctx, api))]
pub(crate) async fn download_post(
    ctx: &impl ctx::Context<'_>,
    api: &API,
    post_id: &str,
    post_title: &str,
    author: &str,
) -> Result<()> {
    let web_name = ctx.web_name();
    let user_id = ctx.user_id();
    let output_dir = ctx.output_dir();

    let whitelist_regex = ctx.whitelist_regexes();
    let blacklist_regex = ctx.blacklist_regexes();
    let whitelist_regex = RegexSet::new(whitelist_regex)?;
    let blacklist_regex = RegexSet::new(blacklist_regex)?;

    let whitelist_filename_regex = ctx.whitelist_filename_regexes();
    let blacklist_filename_regex = ctx.blacklist_filename_regexes();
    let whitelist_filename_regex = RegexSet::new(whitelist_filename_regex)?;
    let blacklist_filename_regex = RegexSet::new(blacklist_filename_regex)?;

    if !whiteblack_regex_filter(&whitelist_regex, &blacklist_regex, post_title) {
        info!("Skipped {post_title} by filter");
        return Ok(());
    }

    let PostInfo {
        post: metadata,
        attachments,
        previews,
    } = api
        .get_post_info(web_name, user_id, post_id)
        .await
        .map_err(|e| anyhow!("failed to get post info: {e}"))?;

    trace!("metadata: {metadata:?}");

    let post_dir = normalize_pathname(post_title);
    let save_path = output_dir.join(&author).join(post_dir.as_str());

    info!("start");

    let attachments = attachments
        .iter()
        .chain(previews.iter())
        .filter_map(|attach| match attach {
            AttachmentLike {
                server: Some(file_server),
                name: Some(file_name),
                path: Some(file_path),
            } if whiteblack_regex_filter(
                &whitelist_filename_regex,
                &blacklist_filename_regex,
                file_name,
            ) =>
            {
                Some(Attachment {
                    file_server,
                    file_name,
                    file_path,
                })
            }
            _ => None,
        });
    download_post_attachments(ctx, &save_path, &api, &metadata, attachments).await?;

    info!("completed");

    Ok(())
}

pub(super) async fn download_post_attachments(
    ctx: &impl ctx::Context<'_>,
    save_path: &PathBuf,
    api: &API,
    metadata: &Post,
    attachments: impl Iterator<Item = Attachment<'_>>,
) -> Result<()> {
    let max_concurrency = ctx.max_concurrency() as u16;

    if DONE.load(Ordering::Relaxed) {
        return Ok(());
    }

    debug!("save_path: {}", save_path.to_string_lossy());
    if let Err(e) = fs::create_dir_all(&save_path).await {
        error!("failed to create save_path: {e}");
        return Ok(());
    };

    let metadata_path = save_path.join("metadata.json");
    debug!("metadata_path: {}", metadata_path.to_string_lossy());

    if let Err(e) = fs::write(
        metadata_path,
        kemono_api::serde_json::to_string_pretty(&metadata)?,
    )
    .await
    {
        error!("failed to write metadata: {e}");
        return Ok(());
    };

    let mut set = HashSet::new();
    let mut tasks = JoinSet::new();

    let (tx, rx) = std::sync::mpmc::channel();

    for Attachment {
        file_server,
        file_name,
        file_path,
    } in attachments
    {
        if DONE.load(Ordering::Relaxed) {
            tasks.join_all().await;
            anyhow::bail!("Received SIGINT, exiting!");
        }

        if !set.insert(file_name) {
            warn!("skipped duplicated file: {file_name}");
            continue;
        }

        let file_url = format!("{file_server}/data{file_path}");
        info!("Downloading {}", file_name);

        let api = api.clone();
        let save_dir = save_path.clone();
        let file_name = file_name.to_string();
        let url = file_url.clone();
        let payload = Payload {
            api,
            url,
            save_dir,
            file_name,
        };
        let _ = tx.send(payload);
    }

    for position in 1..max_concurrency + 1 {
        let rx = rx.clone();
        tasks.spawn(worker(rx, position));
    }

    tasks.join_all().await;

    if DONE.load(Ordering::Relaxed) {
        anyhow::bail!("Received SIGINT, exiting!");
    }

    Ok(())
}
