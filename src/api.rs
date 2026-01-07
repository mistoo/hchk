use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{self, json, Value};
use simple_error::{SimpleError};
use chrono::{DateTime, Utc, TimeZone};
use chrono_humanize::HumanTime;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Check {
    pub id:  Option<String>,
    pub short_id: Option<String>,
    pub name: String,
    pub ping_url: String,
    pub pause_url: String,
    pub last_ping: Option<String>,
    pub next_ping: Option<String>,
    pub grace: u32,
    pub n_pings: u32,
    pub tags: String,
    pub timeout: Option<u32>,
    pub tz:  Option<String>,
    pub schedule: Option<String>,
    pub status: String,
    pub update_url: String
}

fn default_datetime() -> DateTime<Local> {
    let local: DateTime<Local> = Local::now();
    let tz = local.timezone();
    Utc.with_ymd_and_hms(1901, 01, 01, 0, 0, 0).unwrap().with_timezone(&tz)
}

fn parse_datetime(ts: &Option<String>) -> Result<DateTime<Local>, SimpleError> {
    let local: DateTime<Local> = Local::now();
    let tz = local.timezone();

    if ts.is_none() {
        return Ok(default_datetime())
    }

    let ts = ts.as_ref().unwrap();
    let dt = ts.parse::<DateTime<Utc>>()
        .map_err(|e| err(format!("Failed to parse datetime: {}", e)))?;
    Ok(dt.with_timezone(&tz))
}

fn humanize_datetime(dt: DateTime<Local>) -> String {
    return format!("{}", HumanTime::from(dt));
}

impl Check {
    pub fn id(&self) -> String {
        if self.id.is_none() {
            return self.extract_id()
        }

        (&self.id).as_ref().unwrap().to_string()
    }

    pub fn short_id(&self) -> String {
        if self.short_id.is_none() {
            return self.extract_short_id()
        }

        (&self.short_id).as_ref().unwrap().to_string()
    }

    pub fn last_ping_at(&self) -> DateTime<Local> {
        parse_datetime(&self.last_ping).unwrap_or_else(|_| default_datetime())
    }

    pub fn humanized_last_ping_at(&self) -> String {
        // Show "never" for timestamps before 1950 (indicates never pinged)
        let last_ping = self.last_ping_at();
        if last_ping.year() < 1950 {
            return "never".to_string();
        }
        humanize_datetime(last_ping)
    }

    fn fill_ids(&mut self) {
        self.id = Some(self.extract_id());
        self.short_id = Some(self.extract_short_id())
    }

    fn extract_id(&self) -> String {
        let e: Vec<&str> = self.ping_url.rsplitn(2, "/").collect();
        e.first().map(|&id| String::from(id)).unwrap_or_default()
    }

    fn extract_short_id(&self) -> String {
        let id = self.extract_id();
        let e: Vec<&str> = id.splitn(2, "-").collect();
        e.first().map(|&short| String::from(short)).unwrap_or_default()
    }
}

const SECONDS_PER_HOUR: u32 = 3600;
const HOURS_PER_YEAR: u32 = 24 * 365; // 8760 hours (365 days)

fn err(msg: String) -> SimpleError {
    SimpleError::new(msg)
}

pub struct ApiClient {
    client: Client,
    pub base_url: String
}

impl ApiClient {
    pub fn new(base_url: &str, api_key: &str) -> ApiClient {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Api-Key", api_key.parse().unwrap());

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        ApiClient {
            client: client,
            base_url: base_url.parse().unwrap()
        }
    }

    pub fn add(&self, name: &str, schedule: &str, grace: u32, tz: Option<&str>, tags: Option<&str>) -> Result<Check, SimpleError> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(err("Check name cannot be empty".to_string()));
        }
        if grace < 1 || grace > HOURS_PER_YEAR {
            return Err(err(format!("Grace period must be between 1 and {} hours (inclusive)", HOURS_PER_YEAR)));
        }

        let tz_val = tz.unwrap_or("UTC");
        let tags_val = tags.unwrap_or("");

        let c = json!({
            "name":  name,
            "schedule": schedule,
            "grace": grace * SECONDS_PER_HOUR,
            "tags": tags_val,
            "tz": tz_val,
            "unique": [ "name" ]
        });

        let check: Check = self.client
            .post(&self.base_url)
            .json(&c)
            .send()
            .map_err(|e| err(format!("request failed with {:?}", e)))?
            .json()
            .map_err(|e| err(e.to_string()))?;

        Ok(check)
    }


    pub fn delete(&self, check: &Check) -> Result<Check, SimpleError> {
        let url = format!("{}{}", self.base_url, check.id());

        let check: Check = self.client
            .delete(&url)
            .send()
            .map_err(|e| err(format!("request failed with {:?}", e)))?
            .json()
            .map_err(|e| err(e.to_string()))?;

        Ok(check)
    }


    pub fn ping(&self, check: &Check) -> Result<(), SimpleError> {
        self.client
            .get(&check.ping_url)
            .send()
            .map_err(|e| err(format!("request failed with {:?}", e)))?;

        Ok(())
    }


    pub fn pause(&self, check: &Check) -> Result<Check, SimpleError> {
        let url = format!("{}{}/pause", self.base_url, check.id());

        let check: Check = self.client
            .post(&url)
            .send()
            .map_err(|e| err(format! ("request failed with {:?}", e)))?
            .json()
            .map_err(|e| err(e.to_string()))?;

        Ok(check)
    }

    pub fn get(&self, query: Option<&str>) -> Result<Vec<Check>, SimpleError> {
        let v:  Value = self.client
            .get(&self.base_url)
            .send()
            .map_err(|e| err(format!("request failed with {:?}", e)))?
            .json()
            .map_err(|e| err(e.to_string()))?;

        let ref checks_ref = Value::to_string(&v["checks"]);
        let mut checks: Vec<Check> = serde_json::from_str(checks_ref)
            .map_err(|e| err(format!("JSON: {}", e.to_string())))?;

        if let Some(q) = query {
            checks = checks.into_iter().filter(|c| c.name.contains(q) || c.id().contains(q)).collect();
        }

        for c in &mut checks {
            c.fill_ids()
        }

        Ok(checks)
    }


    pub fn find(&self, id: &str) -> Option<Check> {
        let result = self.get(Some(id));
        if result.is_err() {
            eprintln!("Error: {:?}", result);
            return None
        }

        let checks = result.unwrap();
        if checks.is_empty() {
            return None
        }

        checks.first().cloned()
    }
}
