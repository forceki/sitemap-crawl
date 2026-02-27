use reqwest::Client;
use tracing::{warn, error, debug};
use url::Url;

use crate::user_agents::random_user_agent;

pub async fn fetch_page(client: &Client, url: &Url) -> Option<String> {
    let ua = random_user_agent();
    match client.get(url.as_str()).header("User-Agent", ua).send().await {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                warn!(status = %status, url = %url, "Non-success HTTP status");
                return None;
            }
            let content_type = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_lowercase();
            if !content_type.contains("text/html") {
                debug!(url = %url, content_type = %content_type, "Skipping non-HTML");
                return None;
            }
            match resp.text().await {
                Ok(body) => Some(body),
                Err(e) => {
                    error!(url = %url, error = %e, "Failed to read response body");
                    None
                }
            }
        }
        Err(e) => {
            if e.is_timeout() {
                warn!(url = %url, "Request timed out");
            } else if e.is_connect() {
                warn!(url = %url, error = %e, "Connection error");
            } else {
                error!(url = %url, error = %e, "Request failed");
            }
            None
        }
    }
}
