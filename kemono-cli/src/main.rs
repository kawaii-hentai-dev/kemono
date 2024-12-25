use std::{fs, io::IsTerminal, path::PathBuf, sync::atomic::Ordering};

use anyhow::Result;
use clap::Parser;
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use kemono_cli::{
    helper::{batch::download_all, ctx::Args, single::download_one},
    utils::{extract_info, DownloadInfo},
    DONE,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Download tool")]
struct Cli {
    /// kemono URL to fetch posts, can be user profile or single post
    ///
    /// Example:
    ///
    /// https://kemono.su/fanbox/user/4107959
    ///
    /// https://kemono.su/fanbox/user/4107959/post/7999699
    url: String,

    /// Output directory of fetched posts
    #[arg(long, default_value = "./download")]
    output_dir: PathBuf,

    /// Maximium number of tasks running in background concurrently
    #[arg(long, short = 'p', default_value_t = 4)]
    max_concurrency: usize,

    /// Whitelist regex for title
    ///
    /// Specify multiple times means 'AND' semantic
    #[arg(long, short = 'w')]
    whitelist_regex: Vec<String>,

    /// Blacklist regex for title
    ///
    /// Specify multiple times means 'AND' semantic
    #[arg(long, short = 'b')]
    blacklist_regex: Vec<String>,

    /// Whitelist regex for filename
    ///
    /// Specify multiple times means 'AND' semantic
    #[arg(long, short = 'W')]
    whitelist_filename_regex: Vec<String>,

    /// Blacklist regex for filename
    ///
    /// Specify multiple times means 'AND' semantic
    #[arg(long, short = 'B')]
    blacklist_filename_regex: Vec<String>,

    /// Switch to coomer.su endpoint
    #[arg(long, default_value_t = false)]
    coomer: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    kdam::term::init(std::io::stderr().is_terminal());
    kdam::term::hide_cursor()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true)
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .from_env_lossy(),
                ),
        )
        .init();

    let cli = Cli::parse();
    info!("Started with arguments: {cli:?}");
    let Cli {
        url,
        output_dir,
        max_concurrency,
        whitelist_regex,
        blacklist_regex,
        whitelist_filename_regex,
        blacklist_filename_regex,
        coomer,
    } = Cli::parse();

    info!("Download URL: {}", &url);

    fs::create_dir_all(&output_dir)?;

    ctrlc::set_handler(move || {
        if DONE.load(Ordering::Acquire) {
            info!("Signal handler called twice, force-exiting");
            std::process::exit(127);
        } else {
            info!("Signal handler called");
        }
        DONE.store(true, Ordering::Release);
    })?;

    let DownloadInfo {
        web_name,
        user_id,
        post_id,
    } = extract_info(&url)?;

    let args = Args::builder()
        .web_name(web_name)
        .user_id(user_id)
        .max_concurrency(max_concurrency)
        .output_dir(output_dir)
        .whitelist_regexes(whitelist_regex)
        .blacklist_regexes(blacklist_regex)
        .whitelist_filename_regexes(whitelist_filename_regex)
        .blacklist_filename_regexes(blacklist_filename_regex)
        .api_base_url(
            if coomer {
                "https://coomer.su"
            } else {
                "https://kemono.su"
            }
            .into(),
        )
        .build()?;

    match post_id {
        Some(post_id) => {
            if let Err(e) = download_one(&args, &post_id).await {
                error!("{e}");
            }
        }
        None => {
            if let Err(e) = download_all(&args).await {
                error!("{e}");
            }
        }
    }

    info!("Task Exit");

    Ok(())
}
