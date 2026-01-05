use reqwest::blocking::Client;
use serde_json;
use simple_error::{SimpleError};
use serde_json::{Value};
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

fn parse_datetime(ts: &Option<String>) -> DateTime<Local> {
    let local: DateTime<Local> = Local::now();
    let tz = local.timezone();

    if ts.is_none() {
        return Utc.with_ymd_and_hms(1901, 01, 01, 0, 0, 0).unwrap().with_timezone(&tz)
    }

    let ts = ts.clone().unwrap();
    let dt = ts.parse::<DateTime<Utc>>().unwrap();
    dt.with_timezone(&tz)
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
        parse_datetime(&self.last_ping)
    }

    pub fn humanized_last_ping_at(&self) -> String {
        humanize_datetime(self.last_ping_at())
    }

    fn fill_ids(&mut self) {
        self.id = Some(self.extract_id());
        self.short_id = Some(self.extract_short_id())
    }

    fn extract_id(&self) -> String {
        let e: Vec<&str> = self.ping_url.rsplitn(2, "/").collect();
        let id = *e.first().unwrap();
        String::from(id)
    }

    fn extract_short_id(&self) -> String {
        let id = self.extract_id();
        let e: Vec<&str> = id.splitn(2, "-").collect();
        String::from(*e.first().unwrap())
    }
}

//const BASE_URL:  &'static str = "https://healthchecks.io/api/v1/checks/";

fn err(msg: String) -> SimpleError {
    SimpleError::new(msg)
}

pub struct ApiClient {
    client: Client,
    base_url: String
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
        let tz_val = tz.unwrap_or("UTC");
        let tags_val = tags.unwrap_or("");

        let c = json!({
            "name":  name,
            "schedule": schedule,
            "grace": grace * 3600,
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
        let re = self.get(Some(id));
        if re.is_err() {
            println!("err {:?}", re);
            return None
        }

        let checks = re.unwrap();
        if checks.len() == 0 {
            println!("{}: check not found", id);
            return None
        }

        Some((*checks.first().unwrap()).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

    fn sample_check_json() -> String {
        r#"{
            "name": "test-check",
            "ping_url": "https://hc-ping.com/abc123-def456",
            "pause_url": "https://healthchecks.io/api/v1/checks/abc123-def456/pause",
            "last_ping": "2024-01-01T12:00:00+00:00",
            "next_ping": "2024-01-01T13:00:00+00:00",
            "grace": 3600,
            "n_pings": 10,
            "tags": "test",
            "timeout": 86400,
            "tz": "UTC",
            "schedule": "0 * * * *",
            "status": "up",
            "update_url": "https://healthchecks.io/api/v1/checks/abc123-def456"
        }"#.to_string()
    }

    fn sample_checks_response() -> String {
        format!(r#"{{"checks": [{}]}}"#, sample_check_json())
    }

    #[test]
    fn test_check_extract_id() {
        let check = Check {
            id: None,
            short_id: None,
            name: "test".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: None,
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        assert_eq!(check.id(), "abc123-def456");
    }

    #[test]
    fn test_check_extract_short_id() {
        let check = Check {
            id: None,
            short_id: None,
            name: "test".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: None,
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        assert_eq!(check.short_id(), "abc123");
    }

    #[test]
    fn test_check_id_when_already_set() {
        let check = Check {
            id: Some("existing-id".to_string()),
            short_id: None,
            name: "test".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: None,
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        assert_eq!(check.id(), "existing-id");
    }

    #[test]
    fn test_check_short_id_when_already_set() {
        let check = Check {
            id: None,
            short_id: Some("short".to_string()),
            name: "test".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: None,
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        assert_eq!(check.short_id(), "short");
    }

    #[test]
    fn test_parse_datetime_none() {
        let dt = parse_datetime(&None);
        assert_eq!(dt.year(), 1901);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_parse_datetime_valid() {
        let ts = Some("2024-01-15T10:30:00+00:00".to_string());
        let dt = parse_datetime(&ts);
        // Just verify it doesn't panic and returns a valid datetime
        assert!(dt.year() >= 2024);
    }

    #[test]
    fn test_humanize_datetime() {
        let local: DateTime<Local> = Local::now();
        let result = humanize_datetime(local);
        // Should contain "now" or similar humanized text
        assert!(result.len() > 0);
    }

    #[test]
    fn test_check_last_ping_at() {
        let check = Check {
            id: None,
            short_id: None,
            name: "test".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: Some("2024-01-15T10:30:00+00:00".to_string()),
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        let last_ping = check.last_ping_at();
        assert!(last_ping.year() >= 2024);
    }

    #[test]
    fn test_check_humanized_last_ping_at() {
        let check = Check {
            id: None,
            short_id: None,
            name: "test".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: Some("2024-01-15T10:30:00+00:00".to_string()),
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        let humanized = check.humanized_last_ping_at();
        assert!(humanized.len() > 0);
    }

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("https://example.com/api/", "test-api-key");
        assert_eq!(client.base_url, "https://example.com/api/");
    }

    #[test]
    fn test_api_client_add() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/")
            .match_header("X-Api-Key", "test-key")
            .match_body(Matcher::JsonString(r#"{"grace":3600,"name":"test-check","schedule":"0 * * * *","tags":"","tz":"UTC","unique":["name"]}"#.to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(sample_check_json())
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.add("test-check", "0 * * * *", 1, None, None);

        mock.assert();
        assert!(result.is_ok());
        let check = result.unwrap();
        assert_eq!(check.name, "test-check");
        assert_eq!(check.status, "up");
    }

    #[test]
    fn test_api_client_add_with_tags_and_tz() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/")
            .match_header("X-Api-Key", "test-key")
            .match_body(Matcher::JsonString(r#"{"grace":7200,"name":"test-check","schedule":"0 * * * *","tags":"prod,critical","tz":"America/New_York","unique":["name"]}"#.to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(sample_check_json())
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.add("test-check", "0 * * * *", 2, Some("America/New_York"), Some("prod,critical"));

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_client_delete() {
        let mut server = Server::new();
        let mock = server
            .mock("DELETE", "/abc123-def456")
            .match_header("X-Api-Key", "test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(sample_check_json())
            .create();

        let base_url = format!("{}/", server.url());
        let client = ApiClient::new(&base_url, "test-key");
        let check = Check {
            id: Some("abc123-def456".to_string()),
            short_id: Some("abc123".to_string()),
            name: "test-check".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: None,
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        let result = client.delete(&check);
        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_client_ping() {
        let mut server = Server::new();
        let ping_url = format!("{}/ping", server.url());
        
        let mock = server
            .mock("GET", "/ping")
            .with_status(200)
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let check = Check {
            id: Some("abc123-def456".to_string()),
            short_id: Some("abc123".to_string()),
            name: "test-check".to_string(),
            ping_url: ping_url,
            pause_url: "".to_string(),
            last_ping: None,
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        let result = client.ping(&check);
        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_client_pause() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/abc123-def456/pause")
            .match_header("X-Api-Key", "test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "test-check",
                "ping_url": "https://hc-ping.com/abc123-def456",
                "pause_url": "https://healthchecks.io/api/v1/checks/abc123-def456/pause",
                "last_ping": null,
                "next_ping": null,
                "grace": 3600,
                "n_pings": 0,
                "tags": "",
                "timeout": null,
                "tz": null,
                "schedule": null,
                "status": "paused",
                "update_url": "https://healthchecks.io/api/v1/checks/abc123-def456"
            }"#)
            .create();

        let base_url = format!("{}/", server.url());
        let client = ApiClient::new(&base_url, "test-key");
        let check = Check {
            id: Some("abc123-def456".to_string()),
            short_id: Some("abc123".to_string()),
            name: "test-check".to_string(),
            ping_url: "https://hc-ping.com/abc123-def456".to_string(),
            pause_url: "".to_string(),
            last_ping: None,
            next_ping: None,
            grace: 3600,
            n_pings: 0,
            tags: "".to_string(),
            timeout: None,
            tz: None,
            schedule: None,
            status: "up".to_string(),
            update_url: "".to_string(),
        };

        let result = client.pause(&check);
        mock.assert();
        assert!(result.is_ok());
        let paused_check = result.unwrap();
        assert_eq!(paused_check.status, "paused");
    }

    #[test]
    fn test_api_client_get() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/")
            .match_header("X-Api-Key", "test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(sample_checks_response())
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.get(None);

        mock.assert();
        assert!(result.is_ok());
        let checks = result.unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "test-check");
        // Verify that fill_ids was called
        assert!(checks[0].id.is_some());
        assert!(checks[0].short_id.is_some());
    }

    #[test]
    fn test_api_client_get_with_query() {
        let mut server = Server::new();
        let response = r#"{"checks": [
            {
                "name": "test-check-1",
                "ping_url": "https://hc-ping.com/abc123-def456",
                "pause_url": "",
                "last_ping": null,
                "next_ping": null,
                "grace": 3600,
                "n_pings": 0,
                "tags": "",
                "timeout": null,
                "tz": null,
                "schedule": null,
                "status": "up",
                "update_url": ""
            },
            {
                "name": "other-check",
                "ping_url": "https://hc-ping.com/xyz789-ghi012",
                "pause_url": "",
                "last_ping": null,
                "next_ping": null,
                "grace": 3600,
                "n_pings": 0,
                "tags": "",
                "timeout": null,
                "tz": null,
                "schedule": null,
                "status": "up",
                "update_url": ""
            }
        ]}"#;

        let mock = server
            .mock("GET", "/")
            .match_header("X-Api-Key", "test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response)
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.get(Some("test"));

        mock.assert();
        assert!(result.is_ok());
        let checks = result.unwrap();
        // Should only return checks matching "test"
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "test-check-1");
    }

    #[test]
    fn test_api_client_find_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/")
            .match_header("X-Api-Key", "test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(sample_checks_response())
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.find("test-check");

        mock.assert();
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.name, "test-check");
    }

    #[test]
    fn test_api_client_find_not_found() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/")
            .match_header("X-Api-Key", "test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"checks": []}"#)
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.find("nonexistent");

        mock.assert();
        assert!(result.is_none());
    }

    #[test]
    fn test_api_client_find_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/")
            .match_header("X-Api-Key", "test-key")
            .with_status(500)
            .create();

        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.find("test");

        mock.assert();
        assert!(result.is_none());
    }
}
