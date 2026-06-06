# Hello World Pointiv Extension

Rust/WASM template for Pointiv. Greets by name, keeps a run counter, and includes small demos for HTTP, Calendar, and Gmail.

## Install

Paste your GitHub URL in Pointiv Extensions:

```
https://github.com/<your-username>/<your-repo>
```

## Build

Needs [Rust](https://rustup.rs). `./build.sh` writes `extension.wasm` to the repo root. Commit that file so Pointiv can load it from GitHub.

## Try the API demos

Add the permissions you need in `pointiv-extension.json`, rebuild, reinstall.

| Command | Permission | What happens |
|---------|------------|--------------|
| `http` | `network` | GET https://httpbin.org/get, show status and body |
| `calendar` or `cal` | `google_calendar` | Create a test event (title from selection; optional date `YYYY-MM-DD`) |
| `gmail you@example.com` | `google_gmail` | Send mail (body from selection) |

Google commands need Google connected in Pointiv Settings > Account.

Default behavior (any other command): hello + run counter.

## Fork

Your GitHub repo URL is your extension's identity.

1. Edit `pointiv-extension.json` (`name`, `author`, `permissions`)
2. Edit `src/lib.rs`
3. `./build.sh`, commit `extension.wasm`, push

SDK: [pointiv-extension-sdk](https://crates.io/crates/pointiv-extension-sdk)

## Permissions

| Permission | Grants |
|------------|--------|
| `storage` | Key/value store |
| `clipboard_read` | Clipboard |
| `network` | `http::get`, `http::post`, `http::request` |
| `google_calendar` | `google_calendar::schedule` |
| `google_gmail` | `google_gmail::send` |

Calls without permission fail safely (empty data or an error message).
