use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Check {
    pub name: String,
    pub ping_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_id: Option<String>,
    #[serde(skip)]
    cached_id: OnceLock<String>,
    #[serde(skip)]
    cached_short_id: OnceLock<String>,
}

impl Check {
    pub fn id(&self) -> &str {
        if let Some(ref id) = self.id {
            return id;
        }
        self.cached_id.get_or_init(|| self.extract_id())
    }

    pub fn short_id(&self) -> &str {
        if let Some(ref short_id) = self.short_id {
            return short_id;
        }
        self.cached_short_id.get_or_init(|| self.extract_short_id())
    }

    fn extract_id(&self) -> String {
        self.ping_url
            .split('/')
            .last()
            .unwrap_or_default()
            .to_string()
    }

    fn extract_short_id(&self) -> String {
        let id = self.extract_id();
        id.chars().take(8).collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct ChecksResponse {
    pub checks: Vec<Check>,
}

pub async fn fetch_checks(api_key: &str) -> Result<Vec<Check>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = "https://healthchecks.io/api/v3/checks/";

    let response = client
        .get(url)
        .header("X-Api-Key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API request failed: {}", response.status()).into());
    }

    let checks_response: ChecksResponse = response.json().await?;
    Ok(checks_response.checks)
}
