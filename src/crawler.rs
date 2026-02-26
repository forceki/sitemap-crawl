use std::collections::HashSet;
use std::sync::Arc;

use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::{Mutex, Semaphore};
use tracing::{info, error, debug};
use url::Url;

use crate::client::build_client;
use crate::config::DEFAULT_CONCURRENCY;
use crate::extractor::extract_links;
use crate::fetcher::fetch_page;

type VisitedSet = Arc<Mutex<HashSet<String>>>;

pub async fn crawl(start_url: &str) -> Vec<String> {
    let seed = Url::parse(start_url).expect("Invalid start URL");
    let allowed_host = seed
        .host_str()
        .expect("Start URL must have a host")
        .to_string();

    info!(url = %seed, host = %allowed_host, "Starting crawl");

    let client = build_client().expect("Failed to build HTTP client");
    let visited: VisitedSet = Arc::new(Mutex::new(HashSet::new()));
    let semaphore = Arc::new(Semaphore::new(DEFAULT_CONCURRENCY));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Url>();

    {
        let mut set = visited.lock().await;
        set.insert(seed.as_str().to_string());
    }
    tx.send(seed).expect("channel send");

    let mut futures = FuturesUnordered::new();
    let mut active: usize = 0;

    loop {
        while let Ok(url) = rx.try_recv() {
            let client = client.clone();
            let visited = Arc::clone(&visited);
            let sem = Arc::clone(&semaphore);
            let tx = tx.clone();
            let host = allowed_host.clone();

            active += 1;
            futures.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore closed");

                info!(url = %url, "Crawling");

                if let Some(body) = fetch_page(&client, &url).await {
                    let links = extract_links(&body, &url, &host);
                    let new_count;

                    {
                        let mut set = visited.lock().await;
                        let before = set.len();
                        for link in links {
                            let canonical = link.as_str().to_string();
                            if set.insert(canonical) {
                                let _ = tx.send(link);
                            }
                        }
                        new_count = set.len() - before;
                    }

                    if new_count > 0 {
                        debug!(url = %url, new_links = new_count, "Discovered new URLs");
                    }
                }
            }));
        }

        if active == 0 && futures.is_empty() {
            break;
        }

        if let Some(result) = futures.next().await {
            active -= 1;
            if let Err(e) = result {
                error!(error = %e, "Task panicked");
            }
        } else if active == 0 {
            break;
        }
    }

    let set = visited.lock().await;
    let mut urls: Vec<String> = set.iter().cloned().collect();
    urls.sort();

    info!(total = urls.len(), "Crawl complete");
    urls
}
