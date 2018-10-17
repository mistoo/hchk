use ureq;
use serde_json;
use simple_error::{SimpleError};
use serde_json::{Value};
use chrono::{DateTime, Utc, TimeZone};
use chrono_humanize::HumanTime;
use chrono::prelude::*;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Check {
    id: Option<String>,
    short_id: Option<String>,
    pub name: String,
    ping_url: String, //"https://hc-ping.com/662ebe36-ecab-48db-afe3-e20029cb71e6",
    pause_url: String,
    pub last_ping: Option<String>, //DateTime<Utc>, // "2017-01-04T13:24:39.903464+00:00",
    next_ping: Option<String>, //DateTime<Utc>, // "2017-01-04T14:24:39.903464+00:00",
    pub grace: u32, // 900,
    n_pings: u32,
    pub tags: String,
    pub timeout: Option<u32>,
    pub tz: Option<String>,
    pub schedule: Option<String>,
    pub status: String,
    update_url: String
}

fn parse_datetime(ts: &Option<String>) -> DateTime<Local> {
    let local: DateTime<Local> = Local::now();
    let tz = local.timezone();

    if ts.is_none() {
        return Utc.ymd(1901, 01, 01).and_hms(0, 0, 0).with_timezone(&tz)
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
        let e: Vec<&str> = self.ping_url.rsplitn(2, "/").collect();
        String::from(*e.first().unwrap())
    }

    pub fn short_id(&self) -> String {
        let id = self.id();
        let e: Vec<&str> = id.splitn(2, "-").collect();
        String::from(*e.first().unwrap())
    }

    pub fn last_ping_at(&self) -> DateTime<Local> {
        parse_datetime(&self.last_ping)
    }

    pub fn humanized_last_ping_at(&self) -> String {
        humanize_datetime(self.last_ping_at())
    }
}

const BASE_URL: &'static str = "https://healthchecks.io/api/v1/checks";

fn agent(api_key: &str) -> ureq::Agent {
    return ureq::Agent::new().set("X-Api-Key", api_key).build();
}

fn err(msg: String) -> SimpleError  {
    return SimpleError::new(msg)
}

pub fn add_check(api_key: &str, name: String, schedule: String, grace: Option<u32>, tags: Option<String>) -> Result<Check, SimpleError> {
    let c = json!({
        "name": name,
        "schedule": schedule,
        "grace": grace,
        "tags": tags,
        "tz": "Europe/Berlin",
        "unique": [ "name" ]
    });

    let re = agent(api_key).post(BASE_URL).send_json(c);

    if !re.ok() {
        return Err(err(format!("request failed with {:?}", re.status())))
    }

    let reader = re.into_reader();
    let c: Check = serde_json::from_reader(reader).map_err(|e| err(e.to_string()))?;
    return Ok(c)
}

pub fn delete_check(api_key: &str, check: &Check) -> Result<Check, SimpleError> {
    let url = format!("{}/{}", BASE_URL, check.id());
    let re = agent(api_key).delete(url).call();

    if !re.ok() {
        return Err(err(format!("request failed with {:?}", re.status())))
    }

    let reader = re.into_reader();
    let c: Check = serde_json::from_reader(reader).map_err(|e| err(e.to_string()))?;
    return Ok(c)
}


pub fn ping_check(api_key: &str, check: &Check) -> Result<(), SimpleError> {
    let re = agent(api_key).get(check.ping_url.clone()).call();

    if !re.ok() {
        return Err(err(format!("request failed with {:?}", re.status())))
    }
    return Ok(())
}

pub fn pause_check(api_key: &str, check: &Check) -> Result<Check, SimpleError> {
    let url = format!("{}/{}/pause", BASE_URL, check.id());
    let re = agent(api_key).post(url).call();

    if !re.ok() {
        return Err(err(format!("request failed with {:?}", re.status())))
    }

    let reader = re.into_reader();
    let c: Check = serde_json::from_reader(reader).map_err(|e| err(e.to_string()))?;
    return Ok(c)
}

pub fn get_checks(api_key: &str, query: Option<&str>) -> Result<Vec<Check>, SimpleError> {
    let re = agent(api_key).get(BASE_URL).call();

    if !re.ok() {
        return Err(err(format!("request failed with {:?}", re.status())))
    }

    let reader = re.into_reader();
    let v: Value = serde_json::from_reader(reader).map_err(|e| err(e.to_string()))?;


    let ref checks_ref = Value::to_string(&v["checks"]);
    let mut checks: Vec<Check> = serde_json::from_str(checks_ref).map_err(|e| err(format!("JSON: {}", e.to_string())))?;

    if let Some(q) = query {
        checks = checks.into_iter().filter(|c| c.name.contains(q) || c.id().contains(q)).collect();
    }

    return Ok(checks)
}

pub fn find_check(api_key: &str, id: &str) -> Option<Check> {
    let re = get_checks(api_key.clone(), Some(id));
    if re.is_err() {
        println!("err {:?}", re);
        return None
    }

    let checks = re.unwrap();
    if checks.len() == 0 {
        println!("{}: check not found", id);
        return None
    }

    return Some((*checks.first().unwrap()).clone())
}
