// Unit tests for the API module
#[cfg(test)]
mod api_tests {
    use crate::api::*;
    use chrono::prelude::*;
    use mockito::{Matcher, Server};

    fn sample_check_json() -> String {
        r#"{
            "uuid": "abc123-def456",
            "slug": "test-check",
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
        }"#
        .to_string()
    }

    fn sample_checks_response() -> String {
        format!(r#"{{"checks": [{}]}}"#, sample_check_json())
    }

    fn create_test_check(uuid: &str) -> Check {
        Check {
            uuid: uuid.to_string(),
            short_uuid: "".to_string(),
            name: "test".to_string(),
            slug: "test".to_string(),
            ping_url: format!("https://hc-ping.com/{}", uuid),
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
        }
    }

    #[test]
    fn test_check_short_uuid() {
        let mut check = create_test_check("abc123-def456");
        check.set_short_uuid();

        assert_eq!(check.short_uuid, "abc123");
    }

    #[test]
    fn test_check_last_ping_at() {
        let mut check = create_test_check("abc123-def456");
        check.last_ping = Some("2024-01-15T10:30:00+00:00".to_string());

        let last_ping = check.last_ping_at();
        assert!(last_ping.year() >= 2024);
    }

    #[test]
    fn test_check_last_ping_at_none() {
        let check = create_test_check("abc123-def456");

        let last_ping = check.last_ping_at();
        assert_eq!(last_ping.year(), 1901);
        assert_eq!(last_ping.month(), 1);
        assert_eq!(last_ping.day(), 1);
    }

    #[test]
    fn test_check_humanized_last_ping_at() {
        let mut check = create_test_check("abc123-def456");
        check.last_ping = Some("2024-01-15T10:30:00+00:00".to_string());

        let humanized = check.humanized_last_ping_at();
        assert!(humanized.len() > 0);
    }

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("test-key", Some("https://example.com/api/"));
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

        let client = ApiClient::new("test-key", Some(&server.url()));
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

        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.add(
            "test-check",
            "0 * * * *",
            2,
            Some("America/New_York"),
            Some("prod,critical"),
        );

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
        let client = ApiClient::new("test-key", Some(&base_url));
        let check = create_test_check("abc123-def456");

        let result = client.delete(&check);
        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_client_ping() {
        let mut server = Server::new();
        let ping_url = format!("{}/ping", server.url());

        let mock = server.mock("GET", "/ping").with_status(200).create();

        let client = ApiClient::new("test-key", Some(&server.url()));
        //let client = ApiClient::new(&server.url(), "test-key");
        let mut check = create_test_check("abc123-def456");
        check.ping_url = ping_url;

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
            .with_body(
                r#"{
                "uuid": "abc123-def456",
                "name": "test-check",
                "slug": "test-check",
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
            }"#,
            )
            .create();

        let base_url = format!("{}/", server.url());
        let client = ApiClient::new("test-key", Some(&base_url));
        //let client = ApiClient::new(&base_url, "test-key");
        let check = create_test_check("abc123-def456");

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

        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.get(None);

        mock.assert();
        assert!(result.is_ok());
        let checks = result.unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "test-check");
        assert_eq!(checks[0].short_uuid, "abc123");
    }

    #[test]
    fn test_api_client_get_with_query() {
        let mut server = Server::new();
        let response = r#"{"checks": [
            {
                "uuid": "abc123-def456",
                "name": "test-check-1",
                "slug": "test-check-1",
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
                "uuid": "xyz789-ghi012",
                "name": "other-check",
                "slug": "other-check",
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

        let client = ApiClient::new("test-key", Some(&server.url()));
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

        let client = ApiClient::new("test-key", Some(&server.url()));
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

        let client = ApiClient::new("test-key", Some(&server.url()));
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

        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.find("test");

        mock.assert();
        assert!(result.is_none());
    }

    #[test]
    fn test_api_client_add_empty_name() {
        let server = Server::new();
        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.add("", "0 * * * *", 1, None, None);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("name cannot be empty")
        );
    }

    #[test]
    fn test_api_client_add_invalid_grace_zero() {
        let server = Server::new();
        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.add("test", "0 * * * *", 0, None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Grace period"));
    }

    #[test]
    fn test_api_client_add_invalid_grace_too_large() {
        let server = Server::new();
        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.add("test", "0 * * * *", 24 * 366, None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Grace period"));
    }

    #[test]
    fn test_check_humanized_last_ping_never() {
        let mut check = create_test_check("abc123-def456");
        check.short_uuid = "abc123".to_string();

        let humanized = check.humanized_last_ping_at();
        assert_eq!(humanized, "never");
    }

    #[test]
    fn test_api_client_add_unauthorized() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/")
            .match_header("X-Api-Key", "test-key")
            .with_status(401)
            .with_body("Unauthorized")
            .create();

        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.add("test-check", "0 * * * *", 1, None, None);

        mock.assert();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("API error"));
    }

    #[test]
    fn test_api_client_delete_not_found() {
        let mut server = Server::new();
        let mock = server
            .mock("DELETE", "/abc123-def456")
            .match_header("X-Api-Key", "test-key")
            .with_status(404)
            .with_body("Not Found")
            .create();

        let base_url = format!("{}/", server.url());
        let client = ApiClient::new("test-key", Some(&base_url));
        let check = create_test_check("abc123-def456");

        let result = client.delete(&check);
        mock.assert();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("API error"));
    }

    #[test]
    fn test_api_client_ping_server_error() {
        let mut server = Server::new();
        let ping_url = format!("{}/ping", server.url());

        let mock = server
            .mock("GET", "/ping")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        let client = ApiClient::new("test-key", Some(&server.url()));
        let mut check = create_test_check("abc123-def456");
        check.ping_url = ping_url;

        let result = client.ping(&check);
        mock.assert();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("API error"));
    }

    #[test]
    fn test_api_client_pause_forbidden() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/abc123-def456/pause")
            .match_header("X-Api-Key", "test-key")
            .with_status(403)
            .with_body("Forbidden")
            .create();

        let base_url = format!("{}/", server.url());
        let client = ApiClient::new("test-key", Some(&base_url));
        let check = create_test_check("abc123-def456");

        let result = client.pause(&check);
        mock.assert();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("API error"));
    }

    #[test]
    fn test_api_client_get_unauthorized() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/")
            .match_header("X-Api-Key", "test-key")
            .with_status(401)
            .with_body("Unauthorized")
            .create();

        let client = ApiClient::new("test-key", Some(&server.url()));
        let result = client.get(None);

        mock.assert();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("API error"));
    }
}
