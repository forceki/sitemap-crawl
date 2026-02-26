# 🕷️ Sitemap Crawl

A fast, concurrent **Sitemap Crawler & URL Status Checker** built in Rust.

Crawl a website or parse a sitemap XML, then check the HTTP status of every URL and export the results to a timestamped `.xlsx` file.

---

## Features

- **Dual Mode** — crawl by following links _or_ parse a sitemap XML directly
- **Concurrent** — powered by `tokio`, `FuturesUnordered`, and a configurable semaphore
- **Status Checker** — checks every discovered URL for `200`, `301`, `404`, `500`, timeouts, etc.
- **Rate Limiting** — configurable per-request delay to avoid getting blocked
- **Sitemap Index Support** — auto-detects sitemap index files and fetches all child sitemaps
- **Excel Export** — results exported to `.xlsx` with status code, status text, and redirect URL columns
- **CLI Flags** — all settings configurable via `--help`

---

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

### Build

```bash
# Debug build
cargo build

# Optimized release build
cargo build --release
```

The binary will be at `target/release/sitemap-crawl`.

---

## Usage

```
sitemap-crawl [OPTIONS] <URL>
```

### Modes

| Mode | Trigger | Example |
|------|---------|---------|
| **Crawl** | Regular URL | `sitemap-crawl https://example.com` |
| **Sitemap** | URL ending in `.xml` | `sitemap-crawl https://example.com/sitemap.xml` |

### Options

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--concurrency` | `-c` | Max concurrent requests | `50` |
| `--timeout` | `-t` | Request timeout (seconds) | `10` |
| `--delay` | `-d` | Delay between requests per worker (ms) | `100` |
| `--user-agent` | `-u` | Custom User-Agent header | Firefox UA |
| `--output` | `-o` | Output `.xlsx` file path | `result/sitemap_<timestamp>.xlsx` |
| `--help` | `-h` | Show help | — |
| `--version` | `-V` | Show version | — |

### Examples

```bash
# Crawl a website (follows links, stays on same domain)
sitemap-crawl https://example.com

# Parse a product sitemap and check all URLs
sitemap-crawl https://example.com/sitemap-products-1.xml

# Conservative mode: 10 workers, 500ms delay between requests
sitemap-crawl -c 10 -d 500 https://example.com/sitemap-products-1.xml

# Custom output file
sitemap-crawl -o result/exampel.xlsx https://example.com/sitemap-products-1.xml

# Full options
sitemap-crawl -c 20 -t 15 -d 200 -o result/audit.xlsx https://example.com/sitemap.xml
```

---

## Output

### Console

```
Status Check Results
─────────────────────────
  ✅ 2xx OK          : 142
  🔀 3xx Redirect    : 3
  ⚠️  4xx Client Err  : 1
  ❌ 5xx Server Err  : 0
  💀 Connection Err  : 0
  ─────────────────────
  Total              : 146

Exported to result/sitemap_20260226_075300.xlsx
```

### Excel (`.xlsx`)

| No. | URL | Status | Status Text | Redirect URL |
|-----|-----|--------|-------------|--------------|
| 1 | https://example.com/ | 200 | OK | |
| 2 | https://example.com/about | 301 | Moved Permanently | https://example.com/about/ |
| 3 | https://example.com/old-page | 404 | Not Found | |

---

## Project Structure

```
src/
├── main.rs            # Entry point, CLI wiring, output
├── config.rs          # CLI args (clap) & defaults
├── checker.rs         # Concurrent URL status checker
├── client.rs          # HTTP client factory
├── crawler.rs         # BFS crawl engine (follows links)
├── extractor.rs       # HTML link extraction & resolution
├── fetcher.rs         # Async page fetcher with error handling
├── export.rs          # Excel (.xlsx) export
├── sitemap.rs         # Sitemap XML generator
└── sitemap_parser.rs  # Sitemap XML parser (with index support)
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `reqwest` | HTTP client (rustls-tls) |
| `scraper` | HTML parsing |
| `url` | URL resolution & normalization |
| `futures` | `FuturesUnordered` for concurrency |
| `quick-xml` | Sitemap XML parsing |
| `rust_xlsxwriter` | Excel export |
| `clap` | CLI argument parsing |
| `chrono` | Timestamps |

---

## License

MIT
