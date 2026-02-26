mod checker;
mod client;
mod config;
mod crawler;
mod export;
mod extractor;
mod fetcher;
mod sitemap;
mod sitemap_parser;

use checker::check_urls;
use clap::Parser;
use config::AppConfig;
use crawler::crawl;
use export::export_to_xlsx;
use sitemap_parser::parse_sitemap;
use tracing::{info, error, warn};

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
        url = %config.url,
        concurrency = config.concurrency,
        timeout = format!("{}s", config.timeout),
        delay = format!("{}ms", config.delay),
        mode = if config.is_sitemap_xml() { "sitemap" } else { "crawl" },
        "Starting sitemap-crawl"
    );

    let urls = if config.is_sitemap_xml() {
        parse_sitemap(&config.url).await
    } else {
        crawl(&config.url).await
    };

    info!(count = urls.len(), "Discovery complete");

    info!(count = urls.len(), "Checking HTTP status of all URLs");
    let results = check_urls(&urls, &config).await;

    let ok_count = results.iter().filter(|r| matches!(r.status_code, Some(200..=299))).count();
    let redirect_count = results.iter().filter(|r| matches!(r.status_code, Some(300..=399))).count();
    let client_err_count = results.iter().filter(|r| matches!(r.status_code, Some(400..=499))).count();
    let server_err_count = results.iter().filter(|r| matches!(r.status_code, Some(500..=599))).count();
    let error_count = results.iter().filter(|r| r.status_code.is_none()).count();

    info!(
        total = results.len(),
        ok_2xx = ok_count,
        redirect_3xx = redirect_count,
        client_err_4xx = client_err_count,
        server_err_5xx = server_err_count,
        connection_err = error_count,
        "Status check complete"
    );

    if client_err_count > 0 || server_err_count > 0 || error_count > 0 {
        warn!(
            client_err_4xx = client_err_count,
            server_err_5xx = server_err_count,
            connection_err = error_count,
            "Some URLs have issues"
        );

        for r in results.iter().filter(|r| !matches!(r.status_code, Some(200..=299) | Some(300..=399))) {
            warn!(url = %r.url, status = ?r.status_code, reason = %r.status_text, "Problem URL");
        }
    }

    let output_dir = "result/";
    std::fs::create_dir_all(output_dir).expect("Failed to create result/ directory");

    let xlsx_path = config.output.clone().unwrap_or_else(|| {
        let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
        format!("{}/sitemap_{}.xlsx", output_dir, ts)
    });

    match export_to_xlsx(&results, &xlsx_path) {
        Ok(()) => info!(path = %xlsx_path, "Exported results to xlsx"),
        Err(e) => error!(path = %xlsx_path, error = %e, "Failed to export xlsx"),
    }
}
