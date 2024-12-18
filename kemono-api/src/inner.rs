use anyhow::Result;
use reqwest::{Client, Url};

use crate::model::{post_info::PostInfo, posts_legacy::PostsLegacy, user_profile::UserProfile};

#[derive(Clone, Debug)]
pub struct API {
    client: Client,
    base_url: Url,
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 GLS/100.10.9939.100";

impl API {
    pub fn try_new() -> Result<Self> {
        Ok(API {
            client: Client::builder().user_agent(USER_AGENT).build()?,
            base_url: Url::parse("https://kemono.su")?,
        })
    }

    pub fn try_with_base_url(base_url: impl AsRef<str>) -> Result<Self> {
        Ok(API {
            client: Client::builder().user_agent(USER_AGENT).build()?,
            base_url: Url::parse(base_url.as_ref())?,
        })
    }

    pub async fn head(&self, url: &str) -> Result<reqwest::Response> {
        let base_url = &self.base_url;
        let resp = self
            .client
            .head(url)
            .header(reqwest::header::REFERER, base_url.as_str())
            .send()
            .await?;
        Ok(resp)
    }

    pub async fn get_stream(&self, url: &str, start_pos: u64) -> Result<reqwest::Response> {
        let base_url = &self.base_url;
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::REFERER, base_url.as_str())
            .header(reqwest::header::RANGE, format!("bytes={start_pos}-"))
            .send()
            .await?;
        Ok(resp)
    }

    pub async fn get_posts_legacy(
        &self,
        web_name: &str,
        user_id: &str,
        offset: usize,
    ) -> Result<PostsLegacy> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/api/v1/{web_name}/user/{user_id}/posts-legacy",);
        let mut req = self.client.get(&url).header(
            reqwest::header::REFERER,
            format!("{base_url}/{web_name}/user/{user_id}"),
        );

        if offset > 0 {
            req = req.query(&[("o", offset)]);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            return Err(anyhow::anyhow!("GET {url} failed with status {status}",));
        }
        let val = resp.json().await?;
        Ok(val)
    }

    pub async fn get_post_info(
        &self,
        web_name: &str,
        user_id: &str,
        post_id: &str,
    ) -> Result<PostInfo> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/api/v1/{web_name}/user/{user_id}/post/{post_id}");
        let resp = self
            .client
            .get(&url)
            .header(
                reqwest::header::REFERER,
                format!("{base_url}/{web_name}/user/{user_id}/post/{post_id}"),
            )
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            return Err(anyhow::anyhow!("GET {url} failed with status {status}",));
        }
        let val = resp.json().await?;
        Ok(val)
    }

    pub async fn get_user_profile(&self, web_name: &str, user_id: &str) -> Result<UserProfile> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/api/v1/{web_name}/user/{user_id}/profile",);
        let req = self.client.get(&url).header(
            reqwest::header::REFERER,
            format!("{base_url}/{web_name}/user/{user_id}"),
        );

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            return Err(anyhow::anyhow!("GET {url} failed with status {status}",));
        }
        let val = resp.json().await?;
        Ok(val)
    }
}
