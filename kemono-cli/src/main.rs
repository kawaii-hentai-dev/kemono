use std::{
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

use anyhow::Result;
use clap::Parser;
use indicatif::ProgressStyle;
use serde_json::Value;
use tokio::sync::Semaphore;
use tracing::{error, info, level_filters::LevelFilter, warn, Level};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use kemono_api::API;
use kemono_cli::{
    utils::{download_file, extract_info},
    DONE,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Download tool")]
struct Cli {
    url: String,
    #[arg(long)]
    output_dir: Option<PathBuf>,
    #[arg(long, short = 'p')]
    max_concurrency: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let indicatif_layer = IndicatifLayer::new()
        .with_progress_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} {msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                )?
                .progress_chars("#>-"),
        )
        .with_max_progress_bars(u64::MAX, None);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true)
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_filter(LevelFilter::from_level(Level::INFO)),
        )
        .with(indicatif_layer)
        .init();

    let cli = Cli::parse();

    let (web_name, user_id) = extract_info(&cli.url);
    let web_name = match web_name {
        Some(w) => w,
        None => {
            error!("URL Error");
            return Ok(());
        }
    };
    let user_id = match user_id {
        Some(u) => u,
        None => {
            error!("URL Error");
            return Ok(());
        }
    };

    info!("Download URL: {}", &cli.url);

    let output_dir = cli
        .output_dir
        .unwrap_or(std::env::current_dir()?.join("download"));

    fs::create_dir_all(&output_dir)?;

    let api = Arc::new(API::new());

    {
        let done = Arc::clone(&DONE);
        ctrlc::set_handler(move || {
            info!("Signal handler called");
            done.store(true, Ordering::Relaxed);
        })?;
    }

    let mut offset = 0;

    let max_concurrency = cli.max_concurrency.unwrap_or(4);
    let semaphore = Arc::new(Semaphore::new(max_concurrency));

    'outer: loop {
        if DONE.load(Ordering::Relaxed) {
            break;
        }

        let posts_legacy_data = api.get_posts_legacy(&web_name, &user_id, offset).await?;
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
                break 'outer;
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
            if let Err(e) = fs::create_dir_all(&save_path) {
                let es = format!("{}", e);
                if es.contains("267") || es.contains("Invalid argument") {
                    // 这就是 Windows .jpg
                    let new_title = clear_title_re.replace_all(title, "_");
                    save_path = output_dir.join(new_title.as_ref());
                    fs::create_dir_all(&save_path)?;
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

                let api_ref = Arc::clone(&api);
                let sem = Arc::clone(&semaphore);
                let sp = save_path.clone();
                let fname = file_name.to_string();
                let furl = file_url.clone();
                let fut = async move {
                    let _permit = sem.acquire().await;
                    if let Err(e) = download_file(api_ref, &furl, &sp, &fname).await {
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
                    break 'outer;
                }
            }

            if DONE.load(Ordering::Relaxed) {
                break 'outer;
            }
            info!("{} complete", title);
        }

        offset += limit;
        if offset > count {
            break;
        }
    }

    info!("Task Exit");

    Ok(())
}
