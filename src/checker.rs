use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tracing::error;

use crate::config::AppConfig;
use crate::user_agents::random_user_agent;

use rand::Rng;

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

pub async fn check_urls_stream(
    urls: &[String],
    config: &AppConfig,
    tx: mpsc::UnboundedSender<UrlStatus>,
) {
    let client = Client::builder()
        .timeout(config.timeout_duration())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build HTTP client");

    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let delay_ms = config.delay;
    let mut futures = FuturesUnordered::new();

    let completed = Arc::new(AtomicUsize::new(0));

    for url in urls.iter() {
        let client = client.clone();
        let sem = Arc::clone(&semaphore);
        let url = url.clone();
        let completed = Arc::clone(&completed);
        let tx = tx.clone();

        futures.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");

            let random_delay = rand::rng().random_range(delay_ms..=delay_ms * 3);
            tokio::time::sleep(std::time::Duration::from_millis(random_delay)).await;

            let _done = completed.fetch_add(1, Ordering::Relaxed) + 1;

            let ua = random_user_agent();
            let status = match client.get(&url).header("User-Agent", ua).send().await {
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

                    UrlStatus {
                        url,
                        status_code: Some(code),
                        status_text: text,
                        redirect_url,
                    }
                }
                Err(e) => {
                    let text = if e.is_timeout() {
                        "Timeout".to_string()
                    } else if e.is_connect() {
                        "Connection Error".to_string()
                    } else {
                        format!("{}", e)
                    };

                    UrlStatus {
                        url,
                        status_code: None,
                        status_text: text,
                        redirect_url: None,
                    }
                }
            };

            let _ = tx.send(status);
        }));
    }

    while let Some(result) = futures.next().await {
        if let Err(e) = result {
            error!(error = %e, "Task panicked");
        }
    }
}
