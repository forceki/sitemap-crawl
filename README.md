# 🕷️ Sitemap Crawl

A fast, concurrent **Sitemap Crawler & URL Status Checker** built in Rust.

Crawl a website or parse a sitemap XML, then check the HTTP status of every URL and export the results to CSV in real-time.

---

## Features

- **Dual Mode** — crawl by following links _or_ parse a sitemap XML directly
- **Concurrent** — powered by `tokio`, `FuturesUnordered`, and a configurable semaphore
- **Status Checker** — checks every discovered URL for `200`, `301`, `404`, `500`, timeouts, etc.
- **Rate Limiting** — random delay per request to avoid getting blocked
- **User-Agent Rotation** — pool of 20 real browser User-Agents, rotated randomly per request
- **Sitemap Index Support** — auto-detects sitemap index files and fetches all child sitemaps
- **Real-time CSV Export** — results streamed to CSV as they come in
- **Progress Bar** — live progress with speed, ETA, and error count
- **CLI Flags** — all settings configurable via `--help`

---

## Download

Pre-built binaries for Windows, macOS, and Linux are available on the [Releases](https://github.com/forceki/sitemap-crawl/releases) page.

| Platform | File |
|----------|------|
| Windows  | `sitemap-crawl-windows-amd64.zip` |
| macOS (Intel) | `sitemap-crawl-macos-amd64.tar.gz` |
| macOS (Apple Silicon) | `sitemap-crawl-macos-arm64.tar.gz` |
| Linux | `sitemap-crawl-linux-amd64.tar.gz` |

---

## Quick Start

```bash
# Windows
sitemap-crawl.exe https://example.com/sitemap.xml

# macOS / Linux
./sitemap-crawl https://example.com/sitemap.xml

# Multiple sitemaps in one run
./sitemap-crawl https://example.com/sitemap-1.xml https://example.com/sitemap-2.xml
```

---

## Usage

```
sitemap-crawl [OPTIONS] <URL>...
```

You can pass **one or more URLs**. Each URL is auto-detected as crawl or sitemap mode. URLs from all inputs are deduplicated before checking.

### Modes

| Mode | Trigger | Example |
|------|---------|---------|
| **Crawl** | Regular URL | `sitemap-crawl https://example.com` |
| **Sitemap** | URL ending in `.xml` or `.xml.gz` | `sitemap-crawl https://example.com/sitemap.xml` |

### Options

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--concurrency` | `-c` | Max concurrent requests | `50` |
| `--timeout` | `-t` | Request timeout (seconds) | `10` |
| `--delay` | `-d` | Delay between requests per worker (ms) | `100` |
| `--output` | `-o` | Output file path | `result/sitemap_<timestamp>.csv` |
| `--help` | `-h` | Show help | — |
| `--version` | `-V` | Show version | — |

### Examples

```bash
# Single sitemap
sitemap-crawl https://example.com/sitemap.xml

# Multiple sitemaps (all checked in one run, results merged)
sitemap-crawl https://example.com/sitemap-products.xml https://example.com/sitemap-pages.xml

# Gzip-compressed sitemaps
sitemap-crawl https://www.example.com/sitemap-index.xml.gz

# Mix sitemaps and crawl targets
sitemap-crawl https://example.com https://other.com/sitemap.xml

# Crawl a website (follows links, stays on same domain)
sitemap-crawl https://example.com

# Conservative: 10 workers, 500ms delay
sitemap-crawl -c 10 -d 500 https://example.com/sitemap.xml

# Custom output file
sitemap-crawl -o result/my_audit.csv https://example.com/sitemap.xml
```

---

## Output

### Console (Progress Bar)

```
⠹ [00:01:24] [███████████████████▓▒░                   ] 342/1200 (4.1/s) ✅ 330 ❌ 8
  ⚠ https://example.com/old-page [404]
  ✗ https://example.com/broken [Timeout]
```

### CSV (Real-time)

Results are saved to `result/` as they come in:

```
No,URL,Status,Status Text,Redirect URL
1,"https://example.com/",200,"OK",""
2,"https://example.com/about",301,"Moved Permanently","https://example.com/about/"
3,"https://example.com/old-page",404,"Not Found",""
```

You can `tail -f result/sitemap_*.csv` to watch results live, or open with Excel / Google Sheets.

---

## Troubleshooting

### Windows: "Windows protected your PC" (SmartScreen)

This appears because the `.exe` is not digitally signed. To bypass:

1. Click **"More info"**
2. Click **"Run anyway"**

Or run from PowerShell directly:

```powershell
cd C:\Users\YourName\Downloads
.\sitemap-crawl.exe --help
```

### Windows: Antivirus warning

Some antivirus may flag unsigned executables. Add an exception for `sitemap-crawl.exe`, or [build from source](#building-from-source).

### macOS: "Cannot be opened because the developer cannot be verified"

Run once to remove the quarantine flag:

```bash
xattr -d com.apple.quarantine ./sitemap-crawl
```

### Linux: "Permission denied"

```bash
chmod +x ./sitemap-crawl
```

---

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs) (1.70+)

```bash
git clone https://github.com/forceki/sitemap-crawler.git
cd sitemap-crawler
cargo build --release
```

The binary will be at `target/release/sitemap-crawl`.

---

## Project Structure

```
src/
├── main.rs            # Entry point, CLI, progress bar, output
├── config.rs          # CLI args (clap) & defaults
├── checker.rs         # Concurrent URL status checker (streaming)
├── client.rs          # HTTP client factory
├── crawler.rs         # BFS crawl engine (follows links)
├── extractor.rs       # HTML link extraction & resolution
├── fetcher.rs         # Async page fetcher with error handling
├── export.rs          # CSV (real-time) & XLSX export
├── user_agents.rs     # User-Agent rotation pool (20 browsers)
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
| `clap` | CLI argument parsing |
| `indicatif` | Progress bar |
| `tracing` | Structured logging |
| `rand` | Random delay & User-Agent rotation |
| `chrono` | Timestamps |

---

## License

MIT
