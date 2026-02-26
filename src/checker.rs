use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use tokio::sync::Semaphore;
use tracing::{info, warn, error, debug};

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct UrlStatus {
    pub url: String,
    pub status_code: Option<u16>,
    pub status_text: String,
    pub redirect_url: Option<String>,
}

impl fmt::Display for UrlStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status_code {
            Some(code) => {
                write!(f, "[{}] {}", code, self.url)?;
                if let Some(ref redirect) = self.redirect_url {
                    write!(f, " -> {}", redirect)?;
                }
                Ok(())
            }
            None => write!(f, "[ERR] {} ({})", self.url, self.status_text),
        }
    }
}

pub async fn check_urls(urls: &[String], config: &AppConfig) -> Vec<UrlStatus> {
    let client = Client::builder()
        .user_agent(&config.user_agent)
        .timeout(config.timeout_duration())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build HTTP client");

    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let delay = config.delay_duration();
    let mut futures = FuturesUnordered::new();

    let total = urls.len();
    let completed = Arc::new(AtomicUsize::new(0));

    for url in urls.iter() {
        let client = client.clone();
        let sem = Arc::clone(&semaphore);
        let url = url.clone();
        let completed = Arc::clone(&completed);

        futures.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");

            tokio::time::sleep(delay).await;

            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;

            match client.get(&url).send().await {
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    let text = resp.status().canonical_reason()
                        .unwrap_or("Unknown")
                        .to_string();

                    let redirect_url = if resp.status().is_redirection() {
                        resp.headers()
                            .get(reqwest::header::LOCATION)
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string())
                    } else {
                        None
                    };

                    if code >= 400 {
                        warn!(progress = format!("{}/{}", done, total), status = code, url = %url, "HTTP error");
                    } else if code >= 300 {
                        debug!(
                            progress = format!("{}/{}", done, total),
                            status = code,
                            url = %url,
                            redirect = redirect_url.as_deref().unwrap_or("-"),
                            "Redirect"
                        );
                    } else {
                        info!(progress = format!("{}/{}", done, total), status = code, url = %url, "OK");
                    }

                    UrlStatus {
                        url,
                        status_code: Some(code),
                        status_text: text,
                        redirect_url,
                    }
                }
                Err(e) => {
                    let text = if e.is_timeout() {
                        warn!(progress = format!("{}/{}", done, total), url = %url, "Timeout");
                        "Timeout".to_string()
                    } else if e.is_connect() {
                        warn!(progress = format!("{}/{}", done, total), url = %url, "Connection error");
                        "Connection Error".to_string()
                    } else {
                        error!(progress = format!("{}/{}", done, total), url = %url, error = %e, "Request failed");
                        format!("{}", e)
                    };

                    UrlStatus {
                        url,
                        status_code: None,
                        status_text: text,
                        redirect_url: None,
                    }
                }
            }
        }));
    }

    let mut results = Vec::with_capacity(total);

    while let Some(result) = futures.next().await {
        match result {
            Ok(status) => results.push(status),
            Err(e) => error!(error = %e, "Task panicked"),
        }
    }

    results.sort_by(|a, b| a.url.cmp(&b.url));
    results
}
