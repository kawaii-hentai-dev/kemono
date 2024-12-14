use anyhow::Result;
use serde_json::Value;

pub struct API {
    client: reqwest::Client,
}

impl Default for API {
    fn default() -> Self {
        Self::new()
    }
}

impl API {
    pub fn new() -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        API { client }
    }

    pub async fn head(&self, url: &str) -> Result<reqwest::Response> {
        let resp = self
            .client
            .head(url)
            .header("referer", "https://kemono.su")
            .send()
            .await?;
        Ok(resp)
    }

    pub async fn get_stream(&self, url: &str) -> Result<reqwest::Response> {
        let resp = self
            .client
            .get(url)
            .header("referer", "https://kemono.su")
            .send()
            .await?;
        Ok(resp)
    }

    pub async fn get_posts_legacy(
        &self,
        web_name: &str,
        user_id: &str,
        offset: usize,
    ) -> Result<Value> {
        let url = format!(
            "https://kemono.su/api/v1/{}/user/{}/posts-legacy",
            web_name, user_id
        );
        let mut req = self.client.get(&url).header(
            "referer",
            format!("https://kemono.su/{}/user/{}", web_name, user_id),
        );

        if offset > 0 {
            req = req.query(&[("o", offset)]);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "GET {} failed with status {}",
                url,
                resp.status()
            ));
        }
        let val: Value = resp.json().await?;
        Ok(val)
    }

    pub async fn get_post_info(
        &self,
        web_name: &str,
        user_id: &str,
        post_id: &str,
    ) -> Result<Value> {
        let url = format!(
            "https://kemono.su/api/v1/{}/user/{}/post/{}",
            web_name, user_id, post_id
        );
        let resp = self
            .client
            .get(&url)
            .header(
                "referer",
                format!(
                    "https://kemono.su/{}/user/{}/post/{}",
                    web_name, user_id, post_id
                ),
            )
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "GET {} failed with status {}",
                url,
                resp.status()
            ));
        }
        let val: Value = resp.json().await?;
        Ok(val)
    }
}
