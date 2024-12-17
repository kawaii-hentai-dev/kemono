use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use anyhow::{anyhow, Result};
use futures_lite::StreamExt;
use regex::RegexSet;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::{info, info_span, warn};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use kemono_api::{
    reqwest::{self, Url},
    API,
};

use crate::DONE;

/// 提取 web_name 和 user_id
pub fn extract_info(url: &str) -> Result<(String, String)> {
    let url = Url::parse(url)?;
    let mut segments = url
        .path_segments()
        .ok_or_else(|| anyhow!("error: please provide an url with base"))?;
    let web_name = segments
        .next()
        .ok_or_else(|| anyhow!("web_name not found in url"))?;
    if segments.next() != Some("user") {
        anyhow::bail!("wrong url: https://.../<web_name>/user/<user_id>");
    }
    let user_id = segments
        .next()
        .ok_or_else(|| anyhow!("user_id not found in url"))?;

    Ok((web_name.into(), user_id.into()))
}

/// Returns true if passed check
pub fn whiteblack_regex_filter(white: &RegexSet, black: &RegexSet, heytrack: &str) -> bool {
    let white_matched = white.matches(heytrack).matched_all();
    let black_matched = black.matches(heytrack).matched_all();

    match (white_matched, black_matched) {
        _ if white.is_empty() && black.is_empty() => true,
        _ if black.is_empty() => white_matched,
        _ if white.is_empty() => !black_matched,
        (true, false) => true,
        _ => false,
    }
}

pub async fn download_single(api: API, url: &str, save_dir: &Path, file_name: &str) -> Result<()> {
    if DONE.load(Ordering::Relaxed) {
        return Ok(());
    }
    let save_path = save_dir.join(file_name);

    let head_resp = api.head(url).await?;
    if !head_resp.status().is_success() {
        anyhow::bail!(
            "failed to download {} status_code {:?}",
            url,
            head_resp.status()
        );
    }
    let total_size = head_resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    if save_path.exists() && save_path.is_file() {
        let metadata = std::fs::metadata(&save_path)?;
        if metadata.len() == total_size && total_size > 0 {
            warn!("File already exists, skipped {}", file_name);
            return Ok(());
        }
    }

    let span = info_span!("download");
    span.pb_set_message(&format!("Downloading {}", file_name));
    span.pb_set_length(total_size);
    span.pb_start();

    let partial_file_path = save_path.to_string_lossy() + ".incomplete";
    let partial_file_path = PathBuf::from(partial_file_path.as_ref());

    let mut file = match (partial_file_path.exists(), partial_file_path.is_file()) {
        (true, false) => {
            anyhow::bail!("partial_file_path existing as direcotry!");
        }
        _ => {
            File::options()
                .append(true)
                .create(true)
                .open(&partial_file_path)
                .await?
        }
    };

    let start_pos = file.metadata().await?.len();
    let mut pos = start_pos;

    let resp = api.get_stream(url, start_pos).await?;
    if !resp.status().is_success() {
        anyhow::bail!("failed to download {} status_code {:?}", url, resp.status());
    }

    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let data = match item {
            Ok(d) => d,
            Err(e) => {
                // pb.finish_with_message("Error occurred!");
                return Err(e.into());
            }
        };

        if DONE.load(Ordering::Relaxed) {
            drop(file);
            return Ok(());
        }

        file.write_all(&data).await?;
        let len = data.len() as u64;
        pos += len;
        span.pb_set_position(pos);
    }
    file.flush().await?;
    drop(file);
    fs::rename(partial_file_path, save_path).await?;

    // workaround for tracing-indicatif deadlock bu
    // TODO: fix in upstream
    span.pb_finish_clear();
    drop(span);

    info!("Completed downloading {file_name}");
    Ok(())
}
