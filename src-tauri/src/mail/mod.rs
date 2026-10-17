pub mod smtp;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

pub const MAX_EMAILS: usize = 500;

/// A single captured email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedEmail {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub date: String,
    pub body_text: String,
    pub body_html: String,
    pub headers: std::collections::HashMap<String, String>,
    pub size_bytes: usize,
}

impl CapturedEmail {
    pub fn new(
        from: String,
        to: Vec<String>,
        subject: String,
        body_text: String,
        body_html: String,
        headers: std::collections::HashMap<String, String>,
        raw_size: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            subject,
            date: chrono::Local::now().format("%H:%M:%S").to_string(),
            body_text,
            body_html,
            headers,
            size_bytes: raw_size,
        }
    }
}

/// In-memory ring buffer of captured emails (capped at 500)
pub struct MailStore {
    emails: VecDeque<CapturedEmail>,
}

impl MailStore {
    pub fn new() -> Self {
        Self {
            emails: VecDeque::with_capacity(MAX_EMAILS),
        }
    }

    pub fn push(&mut self, email: CapturedEmail) {
        if self.emails.len() >= MAX_EMAILS {
            self.emails.pop_back();
        }
        self.emails.push_front(email);
    }

    pub fn all(&self) -> Vec<CapturedEmail> {
        self.emails.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.emails.clear();
    }

    pub fn count(&self) -> usize {
        self.emails.len()
    }
}
