// Unit tests for the API module
#[cfg(test)]
mod api_tests {
    use crate::api::*;
    use mockito::{Matcher, Server};
    use chrono::prelude::*;
    use std::sync::OnceLock;

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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
        };

        assert_eq!(check.short_id(), "short");
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
        };

        let last_ping = check.last_ping_at();
        assert!(last_ping.year() >= 2024);
    }

    #[test]
    fn test_check_last_ping_at_none() {
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
        };

        let last_ping = check.last_ping_at();
        assert_eq!(last_ping.year(), 1901);
        assert_eq!(last_ping.month(), 1);
        assert_eq!(last_ping.day(), 1);
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
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
        // Verify that IDs are accessible (lazy initialization)
        assert_eq!(checks[0].id(), "abc123-def456");
        assert_eq!(checks[0].short_id(), "abc123");
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

    #[test]
    fn test_api_client_add_empty_name() {
        let server = Server::new();
        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.add("", "0 * * * *", 1, None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name cannot be empty"));
    }

    #[test]
    fn test_api_client_add_invalid_grace_zero() {
        let server = Server::new();
        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.add("test", "0 * * * *", 0, None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Grace period"));
    }

    #[test]
    fn test_api_client_add_invalid_grace_too_large() {
        let server = Server::new();
        let client = ApiClient::new(&server.url(), "test-key");
        let result = client.add("test", "0 * * * *", 24 * 366, None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Grace period"));
    }

    #[test]
    fn test_check_humanized_last_ping_never() {
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
        };

        let humanized = check.humanized_last_ping_at();
        assert_eq!(humanized, "never");
    }

    #[test]
    fn test_check_extract_id_empty_url() {
        let check = Check {
            id: None,
            short_id: None,
            name: "test".to_string(),
            ping_url: "".to_string(),
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
            cached_id: OnceLock::new(),
            cached_short_id: OnceLock::new(),
        };

        // Should not panic with empty URL
        let id = check.id();
        assert_eq!(id, "");
    }
}
