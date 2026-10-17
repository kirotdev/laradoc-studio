use super::{CapturedEmail, MailStore};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

/// Run a minimal SMTP server on the given address.
/// Accepts connections, reads the email DATA, parses it, stores it in MailStore,
/// and emits a `mail:received` Tauri event to update the UI in real-time.
pub async fn run_smtp_server(
    addr: &str,
    store: Arc<Mutex<MailStore>>,
    app: AppHandle,
) -> Result<(), String> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Cannot bind SMTP on {}: {}", addr, e))?;

    log::info!("Local Mail Catcher SMTP server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, peer)) => {
                log::debug!("SMTP connection from {}", peer);
                let store_clone = Arc::clone(&store);
                let app_clone = app.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_smtp_session(socket, store_clone, app_clone).await {
                        log::warn!("SMTP session error from {}: {}", peer, e);
                    }
                });
            }
            Err(e) => log::error!("SMTP accept error: {}", e),
        }
    }
}

async fn handle_smtp_session(
    socket: tokio::net::TcpStream,
    store: Arc<Mutex<MailStore>>,
    app: AppHandle,
) -> Result<(), String> {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    // SMTP greeting
    writer
        .write_all(b"220 laradoc.local Laradoc Studio SMTP Ready\r\n")
        .await
        .map_err(|e| e.to_string())?;

    let mut mail_from = String::new();
    let mut rcpt_to: Vec<String> = Vec::new();
    let mut in_data = false;
    let mut raw_data: Vec<String> = Vec::new();

    while let Ok(Some(line)) = lines.next_line().await {
        let cmd = line.trim().to_uppercase();

        if in_data {
            if line.trim() == "." {
                in_data = false;
                // Parse email data
                let raw = raw_data.join("\n");
                let email = parse_email_data(&raw, &mail_from, &rcpt_to);

                // Store and emit event
                let mut locked = store.lock().await;
                locked.push(email.clone());
                drop(locked);

                if let Err(e) = app.emit("mail:received", &email) {
                    log::warn!("Failed to emit mail:received: {}", e);
                }

                writer.write_all(b"250 OK: Message accepted\r\n").await.ok();
                // Reset for next message in same session
                mail_from.clear();
                rcpt_to.clear();
                raw_data.clear();
            } else {
                // De-dot RFC 5321 DATA
                let line = if line.starts_with("..") { &line[1..] } else { &line };
                raw_data.push(line.to_string());
            }
            continue;
        }

        if cmd.starts_with("EHLO") || cmd.starts_with("HELO") {
            writer
                .write_all(b"250-laradoc.local Hello\r\n250-SIZE 26214400\r\n250 OK\r\n")
                .await
                .ok();
        } else if cmd.starts_with("MAIL FROM:") {
            mail_from = extract_address(&line);
            writer.write_all(b"250 OK\r\n").await.ok();
        } else if cmd.starts_with("RCPT TO:") {
            rcpt_to.push(extract_address(&line));
            writer.write_all(b"250 OK\r\n").await.ok();
        } else if cmd == "DATA" {
            in_data = true;
            writer
                .write_all(b"354 Start input, end with <CRLF>.<CRLF>\r\n")
                .await
                .ok();
        } else if cmd == "QUIT" {
            writer.write_all(b"221 Bye\r\n").await.ok();
            break;
        } else if cmd == "RSET" {
            mail_from.clear();
            rcpt_to.clear();
            raw_data.clear();
            writer.write_all(b"250 OK\r\n").await.ok();
        } else if cmd == "NOOP" {
            writer.write_all(b"250 OK\r\n").await.ok();
        } else {
            writer.write_all(b"502 Command not implemented\r\n").await.ok();
        }
    }

    Ok(())
}

fn extract_address(line: &str) -> String {
    // Extract email from "MAIL FROM:<user@example.com>" or "RCPT TO: user@example.com"
    if let (Some(start), Some(end)) = (line.find('<'), line.rfind('>')) {
        line[start + 1..end].to_string()
    } else {
        line.split(':').last().unwrap_or("").trim().to_string()
    }
}

fn parse_email_data(raw: &str, from: &str, to: &Vec<String>) -> CapturedEmail {
    let mut headers = std::collections::HashMap::new();
    let mut body_start = 0;
    let mut in_headers = true;

    for (i, line) in raw.lines().enumerate() {
        if in_headers {
            if line.is_empty() {
                in_headers = false;
                body_start = i + 1;
                continue;
            }
            if let Some(colon) = line.find(':') {
                let key = line[..colon].trim().to_string();
                let value = line[colon + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }
    }

    let body: Vec<&str> = raw.lines().skip(body_start).collect();
    let body_text = body.join("\n");

    let subject = headers.get("Subject").cloned().unwrap_or_default();
    let content_type = headers.get("Content-Type").cloned().unwrap_or_default();

    let body_html = if content_type.contains("text/html") {
        body_text.clone()
    } else {
        // Convert plain text to minimal HTML
        format!(
            "<pre style=\"font-family: monospace; white-space: pre-wrap;\">{}</pre>",
            html_escape(&body_text)
        )
    };

    CapturedEmail::new(
        from.to_string(),
        to.clone(),
        subject,
        body_text,
        body_html,
        headers,
        raw.len(),
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
