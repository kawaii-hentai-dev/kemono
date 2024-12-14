use std::{fs, path::PathBuf, sync::atomic::Ordering};

use anyhow::Result;
use clap::Parser;
use indicatif::ProgressStyle;
use tracing::{error, info, level_filters::LevelFilter, Level};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use kemono_cli::{batch::download_loop, utils::extract_info, DONE};

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

    let (Some(web_name), Some(user_id)) = extract_info(&cli.url) else {
        error!("URL Error: cannot parse web_name and user_id");
        return Ok(());
    };

    info!("Download URL: {}", &cli.url);

    let output_dir = cli
        .output_dir
        .unwrap_or(std::env::current_dir()?.join("download"));

    fs::create_dir_all(&output_dir)?;

    ctrlc::set_handler(move || {
        info!("Signal handler called");
        DONE.store(true, Ordering::Relaxed);
    })?;

    let max_concurrency = cli.max_concurrency.unwrap_or(4);
    download_loop(web_name, user_id, max_concurrency, &output_dir).await?;

    info!("Task Exit");

    Ok(())
}
