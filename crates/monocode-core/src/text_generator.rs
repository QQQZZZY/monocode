//! Lightweight HTTP client for background text chores (titles, branch names,
//! commit messages, PR descriptions, and next-prompt suggestions) without
//! spawning a heavyweight CLI harness process.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(20);
const SUGGESTION_MAX_CHARS: usize = 200;

/// Wire protocol spoken by the external lightweight text endpoint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextGeneratorKind {
    #[default]
    OpenaiCompatible,
    Anthropic,
}

impl TextGeneratorKind {
    pub const ALL: [Self; 2] = [Self::OpenaiCompatible, Self::Anthropic];

    pub fn id(self) -> &'static str {
        match self {
            Self::OpenaiCompatible => "openai-compatible",
            Self::Anthropic => "anthropic",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.trim() {
            "anthropic" => Self::Anthropic,
            _ => Self::OpenaiCompatible,
        }
    }
}

/// User-configured HTTP endpoint for auxiliary text tasks.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TextGeneratorConfig {
    pub enabled: bool,
    pub kind: TextGeneratorKind,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl TextGeneratorConfig {
    /// Returns the base URL with whitespace/trailing slashes stripped, or the
    /// standard vendor default when left blank.
    pub fn effective_base_url(&self) -> String {
        let trimmed = self.base_url.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
        match self.kind {
            TextGeneratorKind::OpenaiCompatible => "https://api.openai.com/v1".to_string(),
            TextGeneratorKind::Anthropic => "https://api.anthropic.com".to_string(),
        }
    }

    /// True when automatic background generation should use this HTTP config.
    pub fn is_ready(&self) -> bool {
        self.enabled
            && !self.model.trim().is_empty()
            && (!self.base_url.trim().is_empty() || !self.api_key.trim().is_empty())
    }
}

/// Runs a single prompt against the configured HTTP text endpoint when ready.
pub fn generate(
    config: &TextGeneratorConfig,
    system: Option<&str>,
    prompt: &str,
    max_tokens: u32,
    cancel: Arc<AtomicBool>,
) -> Result<String, String> {
    if !config.enabled {
        return Err("HTTP text generator is disabled".into());
    }
    request_completion(config, system, prompt, max_tokens, &cancel)
}

/// Verifies the currently entered settings (regardless of `enabled`) by sending
/// a minimal test completion request.
pub fn test_connection(config: &TextGeneratorConfig) -> Result<String, String> {
    let cancel = AtomicBool::new(false);
    request_completion(
        config,
        Some("Reply with a short confirmation (such as OK)."),
        "OK?",
        16,
        &cancel,
    )
}

/// Generates a suggested follow-up user prompt using the configured HTTP text
/// model from recent conversation turns.
pub fn suggest_next_prompt(
    config: &TextGeneratorConfig,
    recent_exchanges: &[(String, String)],
    cancel: Arc<AtomicBool>,
) -> Option<String> {
    if !config.is_ready() || recent_exchanges.is_empty() {
        return None;
    }
    let prompt = next_prompt_instruction(recent_exchanges);
    let raw = generate(config, Some(NEXT_PROMPT_SYSTEM), &prompt, 96, cancel).ok()?;
    sanitize_prompt_suggestion(&raw)
}

const NEXT_PROMPT_SYSTEM: &str = "Predict the single most likely short follow-up prompt the user would type next in this coding conversation. Output ONLY the prompt text itself with no quotes, markdown fences, labels, or explanations. If no obvious follow-up makes sense, output NONE.";

fn next_prompt_instruction(recent_exchanges: &[(String, String)]) -> String {
    let mut out = String::from("Recent conversation turns:\n");
    for (role, text) in recent_exchanges
        .iter()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let snippet = truncate_tail(text.trim(), 1200);
        if snippet.is_empty() {
            continue;
        }
        out.push_str(role);
        out.push_str(": ");
        out.push_str(&snippet);
        out.push('\n');
    }
    out.push_str("\nNext user prompt:");
    out
}

/// Normalizes a raw model suggestion into a clean single-line ghost prompt.
pub fn sanitize_prompt_suggestion(raw: &str) -> Option<String> {
    let mut text = raw.trim();
    if text.starts_with("```") {
        text = text
            .trim_start_matches('`')
            .split_once('\n')
            .map(|(_, rest)| rest)
            .unwrap_or(text)
            .trim_end_matches('`')
            .trim();
    }
    // Take first non-empty line unless wrapped cleanly.
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?
        .trim();
    let stripped = strip_label_prefix(line)
        .trim()
        .trim_matches(|c| matches!(c, '"' | '\'' | '`'))
        .trim();
    if stripped.is_empty()
        || stripped.eq_ignore_ascii_case("none")
        || stripped.eq_ignore_ascii_case("n/a")
        || stripped.eq_ignore_ascii_case("<none>")
    {
        return None;
    }
    let normalized = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized: String = normalized.chars().take(SUGGESTION_MAX_CHARS).collect();
    (!normalized.is_empty()).then_some(normalized)
}

fn strip_label_prefix(line: &str) -> &str {
    for prefix in [
        "Suggested next prompt:",
        "Suggested prompt:",
        "Next prompt:",
        "Prompt:",
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return rest.trim();
        }
    }
    line
}

fn truncate_tail(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    text.chars().skip(count - max_chars).collect()
}

fn request_completion(
    config: &TextGeneratorConfig,
    system: Option<&str>,
    prompt: &str,
    max_tokens: u32,
    cancel: &AtomicBool,
) -> Result<String, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    let model = config.model.trim();
    if model.is_empty() {
        return Err("Model name is required".into());
    }
    let base_url = config.effective_base_url();
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err("Base URL must start with http:// or https://".into());
    }
    if config.base_url.trim().is_empty() && config.api_key.trim().is_empty() {
        return Err("API key or Base URL is required".into());
    }

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(CONNECT_TIMEOUT)
        .timeout_read(READ_TIMEOUT)
        .build();

    let api_key = config.api_key.trim();
    let result = match config.kind {
        TextGeneratorKind::OpenaiCompatible => {
            let endpoint = openai_endpoint(&base_url);
            let mut messages = Vec::new();
            if let Some(system) = system.filter(|s| !s.trim().is_empty()) {
                messages.push(json!({ "role": "system", "content": system }));
            }
            messages.push(json!({ "role": "user", "content": prompt }));
            let payload = serde_json::to_string(&json!({
                "model": model,
                "messages": messages,
                "max_tokens": max_tokens,
                "temperature": 0.2,
            }))
            .map_err(|err| err.to_string())?;
            let mut req = agent
                .post(&endpoint)
                .set("content-type", "application/json");
            if !api_key.is_empty() {
                req = req.set("authorization", &format!("Bearer {api_key}"));
            }
            req.send_string(&payload)
        }
        TextGeneratorKind::Anthropic => {
            let endpoint = anthropic_endpoint(&base_url);
            let mut payload = json!({
                "model": model,
                "max_tokens": max_tokens,
                "temperature": 0.2,
                "messages": [{ "role": "user", "content": prompt }],
            });
            if let Some(system) = system.filter(|s| !s.trim().is_empty()) {
                payload["system"] = Value::String(system.to_string());
            }
            let body = serde_json::to_string(&payload).map_err(|err| err.to_string())?;
            let mut req = agent
                .post(&endpoint)
                .set("content-type", "application/json")
                .set("anthropic-version", "2023-06-01");
            if !api_key.is_empty() {
                req = req.set("x-api-key", api_key);
            }
            req.send_string(&body)
        }
    };

    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }

    let response = match result {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            return Err(format_http_error(status, &body));
        }
        Err(ureq::Error::Transport(err)) => {
            return Err(format!("HTTP request failed: {err}"));
        }
    };

    let body = response
        .into_string()
        .map_err(|err| format!("Failed reading response: {err}"))?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    let parsed: Value =
        serde_json::from_str(&body).map_err(|err| format!("Invalid JSON response: {err}"))?;

    let text = match config.kind {
        TextGeneratorKind::OpenaiCompatible => extract_openai_text(&parsed),
        TextGeneratorKind::Anthropic => extract_anthropic_text(&parsed),
    };

    text.map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Model returned an empty response".to_string())
}

fn openai_endpoint(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        return base.to_string();
    }
    let after_scheme = base
        .strip_prefix("https://")
        .or_else(|| base.strip_prefix("http://"))
        .unwrap_or(base);
    if !after_scheme.contains('/') {
        format!("{base}/v1/chat/completions")
    } else {
        format!("{base}/chat/completions")
    }
}

fn anthropic_endpoint(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.ends_with("/messages") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/messages")
    } else {
        format!("{base}/v1/messages")
    }
}

fn extract_openai_text(value: &Value) -> Option<String> {
    let content = value
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?;
    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }
    if let Some(parts) = content.as_array() {
        let mut out = String::new();
        for part in parts {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                out.push_str(text);
            }
        }
        if !out.is_empty() {
            return Some(out);
        }
    }
    None
}

fn extract_anthropic_text(value: &Value) -> Option<String> {
    let parts = value.get("content")?.as_array()?;
    let mut out = String::new();
    for part in parts {
        let is_text = part
            .get("type")
            .and_then(Value::as_str)
            .is_none_or(|t| t == "text");
        if is_text {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                out.push_str(text);
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

fn format_http_error(status: u16, body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<Value>(body) {
        if let Some(msg) = json
            .pointer("/error/message")
            .and_then(Value::as_str)
            .or_else(|| json.get("message").and_then(Value::as_str))
            .map(str::trim)
            .filter(|m| !m.is_empty())
        {
            return format!("HTTP {status}: {msg}");
        }
    }
    let snippet: String = body.trim().chars().take(180).collect();
    if snippet.is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {snippet}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn serve_once(status: &str, body: &str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let status = status.to_string();
        let body = body.to_string();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 8192];
            let mut read = Vec::new();
            loop {
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                read.extend_from_slice(&buf[..n]);
                let raw = String::from_utf8_lossy(&read);
                if let Some((head, rest)) = raw.split_once("\r\n\r\n") {
                    let content_len = head
                        .lines()
                        .find_map(|line| {
                            let (k, v) = line.split_once(':')?;
                            k.eq_ignore_ascii_case("content-length")
                                .then(|| v.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .unwrap_or(0);
                    if rest.len() >= content_len {
                        break;
                    }
                }
            }
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            String::from_utf8_lossy(&read).into_owned()
        });
        (format!("http://{addr}"), handle)
    }

    #[test]
    fn openai_compatible_request_and_origin_normalization() {
        let (base_url, handle) = serve_once(
            "200 OK",
            r#"{"choices":[{"message":{"content":"Fix login redirect"}}]}"#,
        );
        let config = TextGeneratorConfig {
            enabled: true,
            kind: TextGeneratorKind::OpenaiCompatible,
            base_url,
            api_key: "sk-test-123".into(),
            model: "gpt-4o-mini".into(),
        };
        assert!(config.is_ready());
        let reply = generate(
            &config,
            Some("Be terse"),
            "Write a commit message",
            64,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(reply, "Fix login redirect");

        let raw_req = handle.join().unwrap();
        assert!(raw_req.starts_with("POST /v1/chat/completions "));
        assert!(raw_req
            .to_ascii_lowercase()
            .contains("authorization: bearer sk-test-123"));
        assert!(raw_req.contains("gpt-4o-mini"));
    }

    #[test]
    fn ollama_local_without_api_key_omits_auth_header() {
        let (base_url, handle) =
            serve_once("200 OK", r#"{"choices":[{"message":{"content":"OK"}}]}"#);
        let config = TextGeneratorConfig {
            enabled: false,
            kind: TextGeneratorKind::OpenaiCompatible,
            base_url: format!("{base_url}/v1"),
            api_key: String::new(),
            model: "qwen2.5:7b".into(),
        };
        assert!(!config.is_ready());
        let reply = test_connection(&config).unwrap();
        assert_eq!(reply, "OK");

        let raw_req = handle.join().unwrap();
        assert!(raw_req.starts_with("POST /v1/chat/completions "));
        assert!(!raw_req.to_ascii_lowercase().contains("authorization:"));
    }

    #[test]
    fn anthropic_request_sends_version_and_api_key_headers() {
        let (base_url, handle) = serve_once(
            "200 OK",
            r#"{"content":[{"type":"text","text":"mc/fix-sidebar"}]}"#,
        );
        let config = TextGeneratorConfig {
            enabled: true,
            kind: TextGeneratorKind::Anthropic,
            base_url,
            api_key: "sk-ant-abc".into(),
            model: "claude-haiku-4-5".into(),
        };
        let reply = generate(
            &config,
            None,
            "Name branch",
            32,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(reply, "mc/fix-sidebar");

        let raw_req = handle.join().unwrap();
        assert!(raw_req.starts_with("POST /v1/messages "));
        let lower = raw_req.to_ascii_lowercase();
        assert!(lower.contains("x-api-key: sk-ant-abc"));
        assert!(lower.contains("anthropic-version: 2023-06-01"));
    }

    #[test]
    fn http_error_surfaces_provider_message_and_cancel_aborts() {
        let (base_url, handle) = serve_once(
            "401 Unauthorized",
            r#"{"error":{"message":"Invalid API key"}}"#,
        );
        let config = TextGeneratorConfig {
            enabled: true,
            kind: TextGeneratorKind::OpenaiCompatible,
            base_url,
            api_key: "bad".into(),
            model: "gpt-4o-mini".into(),
        };
        let err = generate(&config, None, "Hi", 16, Arc::new(AtomicBool::new(false))).unwrap_err();
        assert_eq!(err, "HTTP 401: Invalid API key");
        let _ = handle.join();

        let err = generate(&config, None, "Hi", 16, Arc::new(AtomicBool::new(true))).unwrap_err();
        assert_eq!(err, "Cancelled");
    }

    #[test]
    fn sanitize_prompt_suggestion_filters_wrappers_and_none() {
        assert_eq!(
            sanitize_prompt_suggestion("  \"Run cargo test now\" \nExplanation"),
            Some("Run cargo test now".into())
        );
        assert_eq!(
            sanitize_prompt_suggestion("```text\nSuggested next prompt: Run the build\n```"),
            Some("Run the build".into())
        );
        assert_eq!(sanitize_prompt_suggestion("NONE"), None);
        assert_eq!(sanitize_prompt_suggestion(""), None);
    }

    #[test]
    fn endpoint_helpers_preserve_explicit_paths() {
        assert_eq!(
            openai_endpoint("https://openrouter.ai/api/v1/"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            openai_endpoint("https://ark.cn-beijing.volces.com/api/v3"),
            "https://ark.cn-beijing.volces.com/api/v3/chat/completions"
        );
        assert_eq!(
            openai_endpoint("http://localhost:11434"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            anthropic_endpoint("https://api.anthropic.com/v1"),
            "https://api.anthropic.com/v1/messages"
        );
    }
}
