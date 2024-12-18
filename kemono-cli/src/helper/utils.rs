use anyhow::{anyhow, Result};
use tracing::info;

use kemono_api::{model::user_profile::UserProfile, API};

pub async fn get_author_name(api: &API, web_name: &str, user_id: &str) -> Result<String> {
    let UserProfile {
        ref public_id,
        ref name,
        ..
    } = api
        .get_user_profile(web_name, user_id)
        .await
        .map_err(|e| anyhow!("failed to get user profile: {e}"))?;

    if let Some(public_id) = public_id {
        info!("user ({user_id}): {public_id}");
    }

    Ok(public_id.as_deref().unwrap_or(name).into())
}
