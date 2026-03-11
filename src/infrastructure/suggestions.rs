use std::sync::OnceLock;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SuggestionResult {
    pub text: String,
    pub url: Option<String>,
}

static CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_millis(800))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0")
            .default_headers(
                [(ACCEPT, "application/json".parse().unwrap())]
                    .into_iter()
                    .collect(),
            )
            .build()
            .expect("suggestion http client")
    })
}

pub fn get_suggestions(query: &str) -> Vec<SuggestionResult> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let url = format!(
        "https://suggestqueries.google.com/complete/search?client=firefox&q={}",
        urlencoding::encode(query)
    );

    let resp = match client().get(url).send() {
        Ok(resp) if resp.status().is_success() => resp,
        _ => return Vec::new(),
    };

    let Ok(json): Result<Value, _> = resp.json() else {
        return Vec::new();
    };

    // Google Suggest API (firefox client) returns:
    // [query, [suggestions], [descriptions], [urls]]
    let suggestions = json.get(1).and_then(|v| v.as_array());
    let urls = json.get(3).and_then(|v| v.as_array());

    if let Some(s_arr) = suggestions {
        let mut results = Vec::new();
        for (i, v) in s_arr.iter().enumerate() {
            if let Some(text) = v.as_str() {
                let url = urls
                    .and_then(|u_arr| u_arr.get(i))
                    .and_then(|uv| uv.as_str())
                    .map(|s| s.to_string());

                results.push(SuggestionResult {
                    text: text.to_string(),
                    url,
                });
            }
        }
        results.into_iter().take(7).collect()
    } else {
        Vec::new()
    }
}
