# Hello World Pointiv Extension

Rust/WASM template for Pointiv. Greets by name, keeps a run counter, and includes small demos for HTTP, Calendar, and Gmail, plus a todo list rendered as a popup tile.

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

## Tiles

This starter renders a "Todos" tile beside the popup command bar. `execute` handles the todo commands and `render_tile` (in `src/lib.rs`) turns the stored list into tile JSON.

| Command | What happens |
|---------|--------------|
| `todo add <text>` | Add an item to the list stored under the `todos` storage key |
| `todo done <n>` | Mark item `n` done (1-based, the numbering `todo list` prints) |
| `todo list` | List all items |

The tile shows up to 5 open items, each with a Done button, an open count badge, and a Refresh footer action.

The tile is declared in `pointiv-extension.json`:

```json
"tiles": { "height": 2, "zone": "right", "order": 1 }
```

The host calls `render_tile` when the popup opens and after a tile action runs, with a 3 second budget and storage-only host access. Tile buttons run their command through the normal `execute` function. Iterate with the playground in Pointiv Settings, Tiles: paste tile JSON for instant validation and preview, or live-render this extension's tile after installing it. Full schema, limits, and the component catalog are in TILES.md in the Pointiv repo.

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
