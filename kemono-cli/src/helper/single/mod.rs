use anyhow::Result;
use kemono_api::{
    model::post_info::{Post, PostInfo},
    API,
};

use crate::helper::ctx::Context;

use super::{post::download_post, utils::get_author_name};

pub async fn download_one(ctx: impl Context<'_>, post_id: &str) -> Result<()> {
    let web_name = ctx.web_name();
    let user_id = ctx.user_id();
    let base_url = ctx.api_base_url();

    let api = API::try_with_base_url(base_url)?;

    let author = get_author_name(&api, web_name, user_id).await?;
    let PostInfo {
        post: Post {
            title: post_title, ..
        },
        ..
    } = api.get_post_info(web_name, user_id, post_id).await?;
    let mut post_title = post_title.as_str();
    if post_title.is_empty() {
        post_title = post_id;
    }

    download_post(&ctx, &api, post_id, &post_title, &author).await?;
    Ok(())
}
