use std::{path::PathBuf, sync::mpmc::Receiver};

use kemono_api::API;
use tracing::error;

use crate::utils::download_file;

pub struct Payload {
    pub api: API,
    pub url: String,
    pub save_dir: PathBuf,
    pub file_name: String,
}

pub async fn worker(rx: Receiver<Payload>, position: u16) {
    while let Ok(Payload {
        api,
        url,
        save_dir,
        file_name,
    }) = rx.try_recv()
    {
        if let Err(e) = download_file(api, &url, &save_dir, &file_name, position).await {
            error!("error downloading {file_name}: {e}");
        }
    }
}
