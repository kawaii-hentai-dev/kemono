use std::{fs, path::PathBuf, sync::atomic::Ordering};

use anyhow::Result;
use clap::Parser;
use indicatif::ProgressStyle;
use tracing::{error, info, level_filters::LevelFilter, Level};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use kemono_cli::{
    batch::{ctx::Args, download_loop},
    utils::extract_info,
    DONE,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Download tool")]
struct Cli {
    url: String,
    #[arg(long, default_value = "./download")]
    output_dir: PathBuf,
    #[arg(long, short = 'p', default_value_t = 4)]
    max_concurrency: usize,
    #[arg(long, short = 'w')]
    whitelist_regex: Vec<String>,
    #[arg(long, short = 'b')]
    blacklist_regex: Vec<String>,
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
    info!("Started with arguments: {cli:?}");
    let Cli {
        url,
        output_dir,
        max_concurrency,
        whitelist_regex,
        blacklist_regex,
    } = Cli::parse();

    let (Some(web_name), Some(user_id)) = extract_info(&url) else {
        error!("URL Error: cannot parse web_name and user_id");
        return Ok(());
    };

    info!("Download URL: {}", &url);

    fs::create_dir_all(&output_dir)?;

    ctrlc::set_handler(move || {
        info!("Signal handler called");
        DONE.store(true, Ordering::Relaxed);
    })?;

    let args = Args::builder()
        .web_name(web_name)
        .user_id(user_id)
        .max_concurrency(max_concurrency)
        .output_dir(output_dir)
        .whitelist_regexes(whitelist_regex)
        .blacklist_regexes(blacklist_regex)
        .build()?;
    download_loop(&args).await?;

    info!("Task Exit");

    Ok(())
}
