╔══════════════════════════════════════════════════╗
║         SITEMAP CRAWLER - Quick Start            ║
╚══════════════════════════════════════════════════╝

WHAT IS THIS?
─────────────
A fast command-line tool that crawls a website or reads a sitemap XML,
checks the HTTP status of every URL, and exports the results to a CSV file.


HOW TO RUN
──────────

  Windows:    sitemap-crawl.exe <URL>
  macOS:      ./sitemap-crawl <URL>
  Linux:      ./sitemap-crawl <URL>

Examples:

  sitemap-crawl.exe https://example.com/sitemap.xml
  sitemap-crawl.exe https://example.com
  sitemap-crawl.exe -c 10 -d 500 https://example.com/sitemap.xml


OPTIONS
───────

  -c, --concurrency <N>      Max concurrent requests        (default: 50)
  -t, --timeout <SECS>       Timeout per request in seconds (default: 10)
  -d, --delay <MS>           Delay between requests in ms   (default: 100)
  -o, --output <FILE>        Custom output file path
  -h, --help                 Show all options
  -V, --version              Show version

Full help:  sitemap-crawl.exe --help


OUTPUT
──────

Results are saved to the "result/" folder as a CSV file:

  result/sitemap_20260227_160815.csv

You can open it with Excel, Google Sheets, or any text editor.


═══════════════════════════════════════════════════
 TROUBLESHOOTING
═══════════════════════════════════════════════════

WINDOWS: "Windows protected your PC" (SmartScreen)
──────────────────────────────────────────────────
This warning appears because the .exe is not digitally signed.
It is safe to run. To bypass:

  1. Click "More info"
  2. Click "Run anyway"

Alternatively, run from Command Prompt or PowerShell directly:

  cd C:\Users\YourName\Downloads
  .\sitemap-crawl.exe --help


WINDOWS: "Unrecognized app" or antivirus warning
─────────────────────────────────────────────────
Some antivirus software may flag unsigned executables.
You can add an exception for sitemap-crawl.exe, or build
from source yourself (see below).


macOS: "cannot be opened because the developer cannot be verified"
─────────────────────────────────────────────────────────────────
Run this once to remove the quarantine flag:

  xattr -d com.apple.quarantine ./sitemap-crawl

Then run normally:

  ./sitemap-crawl --help


LINUX: "Permission denied"
──────────────────────────
Make the binary executable first:

  chmod +x ./sitemap-crawl
  ./sitemap-crawl --help


═══════════════════════════════════════════════════
 BUILDING FROM SOURCE
═══════════════════════════════════════════════════

Prerequisites: Rust (https://rustup.rs)

  git clone https://github.com/forceki/sitemap-crawler.git
  cd sitemap-crawler
  cargo build --release

The binary will be at: target/release/sitemap-crawl
