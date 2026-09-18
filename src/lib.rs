//! Hello World sample Pointiv extension (Rust/WASM).
//!
//! Default: greet and count runs in storage.
//! Try these commands in the popup:
//!   `http`      - GET https://httpbin.org/get
//!   `calendar`  - create a test calendar event (title from selected text)
//!   `gmail`     - send email: `gmail to@example.com` (body from selected text)
//!   `todo ...`  - manage the todo list shown in the tile
//!                 (`todo add <text>`, `todo done <n>`, `todo list`)
//!
//! The `render_tile` export below powers the "Todos" tile declared in
//! pointiv-extension.json.

use pointiv_extension_sdk::prelude::*;

/// One todo item, stored under the "todos" storage key as a JSON array.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    text: String,
    done: bool,
}

#[plugin_fn]
pub fn execute(Json(input): Json<Input>) -> FnResult<Json<Output>> {
    let cmd = input.command.trim().to_lowercase();

    if cmd == "todo" || cmd.starts_with("todo ") {
        // Tile actions land here too: clicking "Done" on a tile row runs
        // `todo done <n>` through this same execute function.
        return Ok(Json(todo_command(input.command.trim())));
    }
    if cmd == "http" {
        return Ok(Json(demo_http()));
    }
    if cmd == "calendar" || cmd == "cal" {
        return Ok(Json(demo_calendar(&input)));
    }
    if cmd.starts_with("gmail") || cmd == "email" {
        return Ok(Json(demo_gmail(&input)));
    }

    let count: u64 = storage::read("run_count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        + 1;
    storage::write("run_count", &count.to_string());

    log::info(&format!(
        "execute: count={count}, text_len={}, cmd={:?}",
        input.text.len(),
        input.command,
    ));

    let name = input.text.trim();
    let greeting = if name.is_empty() {
        "Hello, World!".to_string()
    } else {
        format!("Hello, {name}!")
    };

    Ok(Json(Output::text(format!(
        "{greeting}\n\nRun #{count}. Commands: todo add <text>, http, calendar, gmail to@example.com"
    ))))
}

/// GET a public URL and show status + body snippet.
fn demo_http() -> Output {
    let resp = http::get("https://httpbin.org/get");
    if resp.status == 403 {
        return Output::error("network permission not granted. Add \"network\" to pointiv-extension.json.");
    }
    if resp.status == 0 {
        return Output::error("HTTP request failed (host returned no response).");
    }
    let preview: String = resp.body.chars().take(400).collect();
    Output::text(format!("HTTP {}\n\n{preview}", resp.status))
}

/// Create a 30-minute event. Title from selected text. Date from selection if it looks like YYYY-MM-DD.
fn demo_calendar(input: &Input) -> Output {
    let text = input.text.trim();
    let (title, date) = if text.len() == 10 && text.as_bytes().get(4) == Some(&b'-') {
        ("Pointiv test event", text.to_string())
    } else if text.is_empty() {
        ("Pointiv test event", "2026-12-01".to_string())
    } else {
        (text, "2026-12-01".to_string())
    };

    match google_calendar::schedule(
        title,
        &date,
        Some("15:00"),
        Some("15:30"),
        Some("Created by the Hello World example extension"),
    ) {
        Ok(v) => Output::text(format!("Calendar event created.\n\n{v}")),
        Err(e) => Output::error(format!("Calendar failed: {e}")),
    }
}

/// Send mail. Command: `gmail you@example.com`. Body from selected text.
fn demo_gmail(input: &Input) -> Output {
    let parts: Vec<&str> = input.command.split_whitespace().collect();
    let to = parts.get(1).copied().unwrap_or("").trim();
    if to.is_empty() || !to.contains('@') {
        return Output::error(
            "Usage: gmail you@example.com\nPut the email address in the command. Optional body in selected text.",
        );
    }

    let body = input.text.trim();
    let body = if body.is_empty() {
        "Sent from the Pointiv Hello World example extension."
    } else {
        body
    };

    match google_gmail::send(to, "Hello from Pointiv", body) {
        Ok(v) => Output::text(format!("Email sent to {to}.\n\n{v}")),
        Err(e) => Output::error(format!("Gmail failed: {e}")),
    }
}

// ── Todo list + tile ─────────────────────────────────────────────────────────
//
// The todo list demonstrates the tile feature end to end: `execute` mutates
// the list in extension storage, `render_tile` reads the same storage and
// returns a declarative TileUi for the host to draw.

fn load_todos() -> Vec<Todo> {
    storage::read_json("todos").unwrap_or_default()
}

fn save_todos(todos: &[Todo]) {
    storage::write_json("todos", &todos);
}

/// Handle `todo add <text>`, `todo done <n>`, `todo list`.
fn todo_command(command: &str) -> Output {
    // Routing matched the "todo" prefix case-insensitively, so slice it off
    // by length; strip_prefix("todo") would miss e.g. "Todo add milk".
    let rest = command["todo".len()..].trim();
    let (verb, arg) = match rest.split_once(char::is_whitespace) {
        Some((v, a)) => (v, a.trim()),
        None => (rest, ""),
    };

    match verb {
        "add" if !arg.is_empty() => {
            let mut todos = load_todos();
            // Cap stored text so tile rows stay well under the host's
            // 300-char row limit.
            let text: String = arg.chars().take(280).collect();
            todos.push(Todo { text: text.clone(), done: false });
            save_todos(&todos);
            Output::text(format!("Added todo #{}: {text}", todos.len()))
        }
        "done" => {
            // Indices are 1-based positions in the stored array, the same
            // numbering `todo list` prints and the tile rows use.
            let mut todos = load_todos();
            match arg.parse::<usize>() {
                Ok(n) if n >= 1 && n <= todos.len() => {
                    todos[n - 1].done = true;
                    let text = todos[n - 1].text.clone();
                    save_todos(&todos);
                    Output::text(format!("Done: {text}"))
                }
                _ if todos.is_empty() => {
                    Output::error("No todos yet. Add one with: todo add <text>".to_string())
                }
                _ => Output::error(format!("Usage: todo done <n> (1..{})", todos.len())),
            }
        }
        "list" | "" => {
            let todos = load_todos();
            if todos.is_empty() {
                return Output::text("No todos yet. Add one with: todo add <text>".to_string());
            }
            let lines: Vec<String> = todos
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let mark = if t.done { "[x]" } else { "[ ]" };
                    format!("{} {} {}", i + 1, mark, t.text)
                })
                .collect();
            Output::text(lines.join("\n"))
        }
        _ => Output::error("Usage: todo add <text> | todo done <n> | todo list".to_string()),
    }
}

/// Render the "Todos" tile. The host calls this when the popup opens and
/// again after a tile action runs. Only storage host calls are available
/// here, and the render has a 3 second budget.
#[plugin_fn]
pub fn render_tile(Json(_input): Json<TileRenderInput>) -> FnResult<Json<TileUi>> {
    let todos = load_todos();
    let open = todos.iter().filter(|t| !t.done).count();

    // Badge first: open count in warn, or an ok "all done" badge.
    let mut tile = if open > 0 {
        TileUi::new("Todos").badge(format!("{open} open"), TileTone::Warn)
    } else {
        TileUi::new("Todos").badge("all done", TileTone::Ok)
    };

    // Up to 5 not-done rows. Each row's action command carries the item's
    // 1-based index in the stored array, so `todo done <n>` hits the right
    // item even when done items sit between open ones.
    for (index, todo) in todos.iter().enumerate().filter(|(_, t)| !t.done).take(5) {
        // The host rejects the whole tile if any row text exceeds 300 chars,
        // so truncate defensively (older stored todos may predate the add cap).
        let mut text: String = todo.text.chars().take(280).collect();
        if text.len() < todo.text.len() {
            text.push('…');
        }
        tile = tile.row(
            RowBuilder::new(text).action("Done", format!("todo done {}", index + 1)),
        );
    }

    Ok(Json(tile.footer("Refresh", "todo list")))
}
