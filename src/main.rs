mod checker;
mod client;
mod config;
mod crawler;
mod export;
mod extractor;
mod fetcher;
mod sitemap;
mod sitemap_parser;
mod user_agents;

use checker::{check_urls_stream, UrlStatus};
use clap::Parser;
use config::{AppConfig, is_sitemap_url};
use crawler::crawl;
use export::CsvWriter;
use indicatif::{ProgressBar, ProgressStyle};
use sitemap_parser::parse_sitemap;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S".to_string(),
        ))
        .init();

    let config = AppConfig::parse();

    info!(
        urls = config.urls.len(),
        concurrency = config.concurrency,
        timeout = format!("{}s", config.timeout),
        delay = format!("{}ms", config.delay),
        "Starting sitemap-crawl"
    );

    // ── Discover URLs from all inputs ─────────────────────────────────────
    let mut all_discovered: Vec<String> = Vec::new();

    for input_url in &config.urls {
        if is_sitemap_url(input_url) {
            info!(url = %input_url, "Parsing sitemap");
            let urls = parse_sitemap(input_url).await;
            info!(count = urls.len(), url = %input_url, "Found URLs from sitemap");
            all_discovered.extend(urls);
        } else {
            info!(url = %input_url, "Crawling website");
            let urls = crawl(input_url).await;
            info!(count = urls.len(), url = %input_url, "Found URLs from crawl");
            all_discovered.extend(urls);
        }
    }

    // Deduplicate across all inputs
    all_discovered.sort();
    all_discovered.dedup();

    info!(count = all_discovered.len(), "Total unique URLs to check");

    // ── Prepare output ────────────────────────────────────────────────────
    let output_dir = "result/";
    std::fs::create_dir_all(output_dir).expect("Failed to create result/ directory");

    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let csv_path = config.output.clone().unwrap_or_else(|| {
        format!("{}sitemap_{}.csv", output_dir, ts)
    });

    let mut csv_writer = CsvWriter::new(&csv_path).expect("Failed to create CSV writer");
    info!(path = %csv_path, "Streaming results to CSV");

    // ── Check all URLs with progress bar ──────────────────────────────────
    let total = all_discovered.len();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UrlStatus>();

    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}) {msg}"
        )
        .unwrap()
        .progress_chars("█▓▒░  "),
    );
    pb.set_message("Checking URLs...");

    let check_config = config.clone();
    let check_handle = tokio::spawn(async move {
        check_urls_stream(&all_discovered, &check_config, tx).await;
    });

    let mut all_results: Vec<UrlStatus> = Vec::with_capacity(total);
    let mut ok_count: usize = 0;
    let mut err_count: usize = 0;

    while let Some(status) = rx.recv().await {
        csv_writer.append_row(&status).expect("Failed to write CSV row");

        match status.status_code {
            Some(200..=299) => ok_count += 1,
            Some(code) if code >= 400 => {
                err_count += 1;
                pb.println(format!("  ⚠ {} [{}]", status.url, code));
            }
            None => {
                err_count += 1;
                pb.println(format!("  ✗ {} [{}]", status.url, status.status_text));
            }
            _ => {}
        }

        all_results.push(status);
        pb.set_message(format!("✅ {} ❌ {}", ok_count, err_count));
        pb.inc(1);
    }

    check_handle.await.expect("Checker task panicked");
    pb.finish_with_message(format!("Done — ✅ {} ❌ {}", ok_count, err_count));

    // ── Summary ───────────────────────────────────────────────────────────
    info!(path = %csv_path, rows = csv_writer.row_count(), "CSV export complete");

    let ok_count = all_results.iter().filter(|r| matches!(r.status_code, Some(200..=299))).count();
    let redirect_count = all_results.iter().filter(|r| matches!(r.status_code, Some(300..=399))).count();
    let client_err_count = all_results.iter().filter(|r| matches!(r.status_code, Some(400..=499))).count();
    let server_err_count = all_results.iter().filter(|r| matches!(r.status_code, Some(500..=599))).count();
    let error_count = all_results.iter().filter(|r| r.status_code.is_none()).count();

    info!(
        total = all_results.len(),
        ok_2xx = ok_count,
        redirect_3xx = redirect_count,
        client_err_4xx = client_err_count,
        server_err_5xx = server_err_count,
        connection_err = error_count,
        "Status check complete"
    );
}
