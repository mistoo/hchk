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

const BASE_URL:  &'static str = "https://healthchecks.io/api/v1/checks/";

fn client(api_key: &str) -> Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Api-Key", api_key.parse().unwrap());

    Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

fn err(msg: String) -> SimpleError {
    SimpleError::new(msg)
}

pub fn add_check(api_key: &str, name: &str, schedule: &str, grace: u32, tz: Option<&str>, tags: Option<&str>) -> Result<Check, SimpleError> {
    let tz_val = tz.unwrap_or("UTC");
    let tags_val = tags.unwrap_or("");

    // shorter form ("* * * * *") is not supported by Schedule
    //let schedul = Schedule::from_str(schedule);
    //if schedul.is_err() {
    //    return Err(err(format!("schedule parse error {:?}", schedule)))
    //}

    let c = json!({
        "name":  name,
        "schedule": schedule,
        "grace": grace * 3600,
        "tags": tags_val,
        "tz": tz_val,
        "unique": [ "name" ]
    });

    let check: Check = client(api_key)
        .post(BASE_URL)
        .json(&c)
        .send()
        .map_err(|e| err(format!("request failed with {:?}", e)))?
        .json()
        .map_err(|e| err(e.to_string()))?;

    Ok(check)
}

pub fn delete_check(api_key: &str, check: &Check) -> Result<Check, SimpleError> {
    let url = format!("{}{}", BASE_URL, check.id());

    let check: Check = client(api_key)
        .delete(&url)
        .send()
        .map_err(|e| err(format!("request failed with {:?}", e)))?
        .json()
        .map_err(|e| err(e.to_string()))?;

    Ok(check)
}

pub fn ping_check(api_key: &str, check: &Check) -> Result<(), SimpleError> {
    client(api_key)
        .get(&check.ping_url)
        .send()
        .map_err(|e| err(format!("request failed with {:?}", e)))?;

    Ok(())
}

pub fn pause_check(api_key: &str, check: &Check) -> Result<Check, SimpleError> {
    let url = format!("{}{}/pause", BASE_URL, check.id());

    let check: Check = client(api_key)
        .post(&url)
        .send()
        .map_err(|e| err(format! ("request failed with {:?}", e)))?
        .json()
        .map_err(|e| err(e.to_string()))?;

    Ok(check)
}

pub fn get_checks(api_key: &str, query: Option<&str>) -> Result<Vec<Check>, SimpleError> {
    let v:  Value = client(api_key)
        .get(BASE_URL)
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

pub fn find_check(api_key:  &str, id: &str) -> Option<Check> {
    let re = get_checks(api_key, Some(id));
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
