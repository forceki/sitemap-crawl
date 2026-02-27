use flate2::read::GzDecoder;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Client;
use std::io::Read;
use tracing::{info, warn, error};
use url::Url;

use crate::client::build_client;
use crate::user_agents::random_user_agent;

pub async fn parse_sitemap(sitemap_url: &str) -> Vec<String> {
    let client = build_client().expect("Failed to build HTTP client");

    info!(url = %sitemap_url, "Downloading sitemap");

    let xml = match fetch_xml(&client, sitemap_url).await {
        Some(body) => body,
        None => {
            error!(url = %sitemap_url, "Failed to fetch sitemap XML");
            return Vec::new();
        }
    };

    let urls = extract_locs(&xml);

    if urls.iter().all(|u| u.ends_with(".xml") || u.ends_with(".xml.gz")) {
        let mut all_urls = Vec::new();

        for child_sitemap in urls.iter() {
            if let Some(child_xml) = fetch_xml(&client, child_sitemap).await {
                let child_urls = extract_locs(&child_xml);
                info!(count = child_urls.len(), url = %child_sitemap, "Parsed child sitemap");
                all_urls.extend(child_urls);
            }
        }

        all_urls.sort();
        all_urls.dedup();
        info!(count = all_urls.len(), "Total unique URLs from sitemap index");
        all_urls
    } else {
        info!(count = urls.len(), "Parsed sitemap URLs");
        let mut sorted = urls;
        sorted.sort();
        sorted.dedup();
        sorted
    }
}

async fn fetch_xml(client: &Client, url: &str) -> Option<String> {
    let ua = random_user_agent();
    let is_gz = url.ends_with(".gz");

    match client.get(url).header("User-Agent", ua).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                warn!(status = %resp.status(), url = %url, "HTTP error fetching sitemap");
                return None;
            }

            if is_gz {
                // Fetch as raw bytes and decompress gzip
                match resp.bytes().await {
                    Ok(bytes) => {
                        info!(url = %url, compressed_bytes = bytes.len(), "Downloaded .gz sitemap");
                        match decompress_gz(&bytes) {
                            Ok(text) => Some(text),
                            Err(e) => {
                                error!(url = %url, error = %e, "Failed to decompress .gz");
                                None
                            }
                        }
                    }
                    Err(e) => {
                        error!(url = %url, error = %e, "Failed to read response bytes");
                        None
                    }
                }
            } else {
                // Plain XML
                match resp.text().await {
                    Ok(body) => Some(body),
                    Err(e) => {
                        error!(url = %url, error = %e, "Failed to read response body");
                        None
                    }
                }
            }
        }
        Err(e) => {
            error!(url = %url, error = %e, "Request failed");
            None
        }
    }
}

fn decompress_gz(data: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut xml = String::new();
    decoder.read_to_string(&mut xml)?;
    Ok(xml)
}

fn extract_locs(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    let mut urls = Vec::new();
    let mut inside_loc = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"loc" => {
                inside_loc = true;
            }
            Ok(Event::Text(ref e)) if inside_loc => {
                if let Ok(text) = e.unescape() {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        if Url::parse(&trimmed).is_ok() {
                            urls.push(trimmed);
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"loc" => {
                inside_loc = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!(error = %e, "XML parse error");
                break;
            }
            _ => {}
        }
    }

    urls
}
