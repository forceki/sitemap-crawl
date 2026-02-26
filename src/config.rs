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
    long_about = "Crawl a website or parse a sitemap XML, then check HTTP status of every URL.\n\nModes:\n  • Crawl mode   — pass a regular URL (e.g. https://example.com)\n  • Sitemap mode — pass a .xml URL (e.g. https://example.com/sitemap.xml)\n\nResults are exported to a timestamped .xlsx file with color-coded status.",
    version
)]
pub struct AppConfig {
    /// Target URL to crawl or sitemap XML URL to parse
    pub url: String,

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

    /// Output xlsx file name (default: sitemap_<timestamp>.xlsx)
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

    pub fn is_sitemap_xml(&self) -> bool {
        self.url.ends_with(".xml") || self.url.ends_with(".xml.gz")
    }
}
