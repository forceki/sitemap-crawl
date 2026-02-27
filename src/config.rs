use std::time::Duration;
use clap::Parser;

pub const DEFAULT_CONCURRENCY: usize = 50;
pub const DEFAULT_TIMEOUT: u64 = 10;
pub const DEFAULT_DELAY: u64 = 100;
pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.6; rv:37.0) Gecko/20100101 Firefox/37.0";

#[derive(Parser, Debug, Clone)]
#[command(
    name = "sitemap-crawl",
    about = "Concurrent Sitemap Crawler & URL Status Checker",
    long_about = "Crawl websites or parse sitemap XMLs, then check HTTP status of every URL.\n\nSupports multiple URLs in a single run. Each URL is auto-detected as crawl or sitemap mode.\n\nExamples:\n  sitemap-crawl https://example.com\n  sitemap-crawl https://example.com/sitemap.xml\n  sitemap-crawl https://a.com/sitemap.xml https://b.com/sitemap.xml\n  sitemap-crawl https://a.com/s1.xml https://a.com/s2.xml.gz",
    version
)]
pub struct AppConfig {
    /// One or more URLs to crawl or sitemap XMLs to parse
    #[arg(required = true)]
    pub urls: Vec<String>,

    /// Max concurrent requests
    #[arg(short, long, default_value_t = DEFAULT_CONCURRENCY)]
    pub concurrency: usize,

    /// Request timeout in seconds
    #[arg(short, long, default_value_t = DEFAULT_TIMEOUT)]
    pub timeout: u64,

    /// Delay between requests in milliseconds (per worker, to avoid rate limiting)
    #[arg(short, long, default_value_t = DEFAULT_DELAY)]
    pub delay: u64,

    /// Custom User-Agent header
    #[arg(short, long, default_value = DEFAULT_USER_AGENT)]
    pub user_agent: String,

    /// Output file name (default: result/sitemap_<timestamp>.csv)
    #[arg(short, long)]
    pub output: Option<String>,
}

impl AppConfig {
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }

    pub fn delay_duration(&self) -> Duration {
        Duration::from_millis(self.delay)
    }
}

pub fn is_sitemap_url(url: &str) -> bool {
    url.ends_with(".xml") || url.ends_with(".xml.gz")
}
