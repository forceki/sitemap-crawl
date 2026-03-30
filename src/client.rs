use reqwest::{Client, Response};
use std::time::Duration;
use tracing::warn;

use crate::config::DEFAULT_TIMEOUT;
use crate::user_agents::random_user_agent;

pub fn build_client(proxy: Option<&str>) -> reqwest::Result<Client> {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT));
        
    if let Some(p) = proxy {
        builder = builder.proxy(reqwest::Proxy::all(p)?);
    }
    
    builder.build()
}

pub async fn get_with_retry(client: &Client, url: &str, max_retries: u32) -> Result<Response, reqwest::Error> {
    let mut retries = 0;
    let mut backoff_sec = 2;

    loop {
        let ua = random_user_agent();
        let resp_result = client.get(url).header("User-Agent", ua).send().await;

        match resp_result {
            Ok(resp) => {
                let status = resp.status();
                // Check for 429 Too Many Requests or 5xx Server Errors
                if (status == 429 || status.is_server_error()) && retries < max_retries {
                    // Try to respect Retry-After header
                    let retry_after = resp
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(backoff_sec);

                    // warn!(
                    //     url = %url,
                    //     status = %status,
                    //     retry = retries + 1,
                    //     retry_after_sec = retry_after,
                    //     "Rate limited or server error, retrying..."
                    // );
                    tokio::time::sleep(Duration::from_secs(retry_after)).await;
                    retries += 1;
                    backoff_sec *= 2; // exponential backoff
                    continue;
                }
                return Ok(resp);
            }
            Err(e) => {
                if retries < max_retries && (e.is_timeout() || e.is_connect()) {
                    warn!(
                        url = %url,
                        error = %e,
                        retry = retries + 1,
                        retry_after_sec = backoff_sec,
                        "Network error, retrying..."
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_sec)).await;
                    retries += 1;
                    backoff_sec *= 2;
                    continue;
                }
                return Err(e);
            }
        }
    }
}
