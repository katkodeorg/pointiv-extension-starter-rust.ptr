//! Hello World sample Pointiv extension (Rust/WASM).
//!
//! Default: greet and count runs in storage.
//! Try these commands in the popup:
//!   `http`     - GET https://httpbin.org/get
//!   `calendar` - create a test calendar event (title from selected text)
//!   `gmail`    - send email: `gmail to@example.com` (body from selected text)

use pointiv_extension_sdk::prelude::*;

#[plugin_fn]
pub fn execute(Json(input): Json<Input>) -> FnResult<Json<Output>> {
    let cmd = input.command.trim().to_lowercase();

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
        "{greeting}\n\nRun #{count}. Commands: http, calendar, gmail to@example.com"
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
