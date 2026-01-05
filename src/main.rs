extern crate simple_error;
extern crate chrono;
extern crate chrono_tz;
extern crate chrono_humanize;
extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate clap;
extern crate colored;
extern crate isatty;

use std::env;
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use clap::{Arg, App, SubCommand};
use colored::*;
use isatty::{stdout_isatty};

mod api;
use crate::api::ApiClient;

#[cfg(test)]
mod tests;

const BASE_URL: &'static str = "https://healthchecks.io/api/v1/checks/";

fn colored_status(status: &str) -> ColoredString {
    match status {
        "up" => status.green(),
        "down" => status.red(),
        "grace" => status.cyan(),
        "paused" => status.yellow(),
        _ => status.white(),
    }
}

struct LsFlags {
    up: bool,
    down: bool,
    long: bool
}

fn cmd_list_checks(client: &ApiClient, flags: &LsFlags, query: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let result = client.get(query)?;

    let mut checks = result;

    checks.sort_by(|a, b| a.name.cmp(&b.name));
    if flags.up || flags.down {
        checks = checks.into_iter().filter(|c| (flags.down && c.status == "down") || (flags.up && c.status == "up")).collect();
    }

    let tty = stdout_isatty();
    if tty {
        println!("total {:?}", checks.len());
    }

    for c in checks {
        if flags.long {
            println!("{}", serde_json::to_string_pretty(&c)?);
            continue
        }

        let mut status = colored_status(&c.status);
        if !tty {
            status = status.clear();
        }

        println!("{status:<s_width$} {id:<id_width$} {name:<n_width$} {last_ping:<lp_width$}",
                 name=c.name,
                 status=status,
                 id=c.short_id(),
                 last_ping=c.humanized_last_ping_at(),
                 s_width=6, id_width=9, n_width=40, lp_width=30);
    }

    Ok(())
}

fn cmd_add_check(client: &ApiClient, name: Option<&str>, schedule: Option<&str>, grace: Option<&str>, tz: Option<&str>, tags: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let name = name.ok_or("Name is required")?;
    let schedule = schedule.ok_or("Schedule is required")?;
    
    let grace_s = grace.unwrap_or("1");
    let grace_v = grace_s.parse::<u32>()
        .map_err(|_| format!("Invalid grace period: {}", grace_s))?;

    let check = client.add(name, schedule, grace_v, tz, tags)?;
    println!("{} {} {}", check.name, check.id(), check.ping_url);
    
    Ok(())
}

fn cmd_pause_check(client: &ApiClient, id: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let id = id.ok_or("ID is required")?;
    
    let c = client.find(id)
        .ok_or_else(|| format!("{}: check not found", id))?;

    if c.status == "paused" {
        println!("{}: check is already paused", c.name);
        return Ok(());
    }

    client.pause(&c)?;
    Ok(())
}

fn cmd_ping_check(client: &ApiClient, id: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let id = id.ok_or("ID is required")?;
    
    let c = client.find(id)
        .ok_or_else(|| format!("{}: check not found", id))?;
    
    client.ping(&c)?;
    Ok(())
}

fn cmd_delete_check(client: &ApiClient, id: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let id = id.ok_or("ID is required")?;
    
    let c = client.find(id)
        .ok_or_else(|| format!("{}: check not found", id))?;
    
    client.delete(&c)?;
    Ok(())
}

fn keyfile_path() -> String {
    let home = env::var("HOME");
    if home.is_err() {
        println!("empty HOME environment variable");
        process::exit(1);
    }

    home.unwrap() + "/.hchk"
}

fn cmd_setkey(key: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let key = key.ok_or("API key is required")?;
    
    let path = keyfile_path();
    let mut file = File::create(&path)?;
    file.write_all(key.as_bytes())?;
    
    // Set file permissions to 0o600 (read/write for owner only) on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, permissions)?;
    }
    
    Ok(())
}

const API_KEY_ENV: &'static str = "HCHK_API_KEY";
fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let key = env::var(API_KEY_ENV);

    if key.is_err() {
        let path = keyfile_path();
        if Path::new(&path).is_file() {
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            return Ok(contents);
        }
    }

    if key.is_err() {
        return Err(format!("Use setkey command or set {} environment variable", API_KEY_ENV).into());
    }

    Ok(key.unwrap())
}

enum Command {
    List,
    Add,
    Delete,
    Pause,
    Ping,
    SetKey
}

fn run(cmd: Command, args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let key = match cmd {
        Command::SetKey => "".to_string(),
        _ => get_api_key()?
    };

    let client = ApiClient::new(BASE_URL, &key);

    match cmd {
        Command::List => cmd_list_checks(&client,
                                         &LsFlags{ long: args.is_present("long"),
                                                   up: args.is_present("up"),
                                                   down: args.is_present("down") },
                                         args.value_of("query")),
        Command::Add => cmd_add_check(&client,
                                      args.value_of("name"),
                                      args.value_of("schedule"),
                                      args.value_of("grace"),
                                      args.value_of("tz"),
                                      args.value_of("tags")),
        Command::Ping => cmd_ping_check(&client, args.value_of("id")),
        Command::Pause => cmd_pause_check(&client, args.value_of("id")),
        Command::Delete => cmd_delete_check(&client, args.value_of("id")),
        Command::SetKey => cmd_setkey(args.value_of("key"))
    }
}

fn main() {
    let matches = App::new("hchk")
        .version("0.1.0")
        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("be verbose"))
        .subcommand(SubCommand::with_name("setkey").about("Save API key to $HOME/.hchk")
                    .arg(Arg::with_name("key").help("API key")))
        .subcommand(SubCommand::with_name("ls").about("List checks")
                    .arg(Arg::with_name("long").short("l").help("long listing"))
                    .arg(Arg::with_name("up").short("u").help("list 'up' only checks"))
                    .arg(Arg::with_name("down").short("d").help("list 'down' only checks"))
                    .arg(Arg::with_name("query").help("filter by name/id")))
        .subcommand(SubCommand::with_name("pause").about("Pause check")
                    .arg(Arg::with_name("id").help("check's ID to pause").required(true)))
        .subcommand(SubCommand::with_name("ping").about("Ping check")
                    .arg(Arg::with_name("id").help("check's ID to ping").required(true)))
        .subcommand(SubCommand::with_name("del").about("Delete check")
                    .arg(Arg::with_name("id").help("check's ID to delete").required(true)))
        .subcommand(SubCommand::with_name("add").about("Add check")
                    .arg(Arg::with_name("name").help("name").required(true))
                    .arg(Arg::with_name("schedule").help("schedule in cron format").required(true))
                    .arg(Arg::with_name("grace").help("grace in hours"))
                    .arg(Arg::with_name("tz").help("timezone"))
                    .arg(Arg::with_name("tags").help("tags")))

        .get_matches();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    let result = if let Some(matches) = matches.subcommand_matches("setkey") {
        run(Command::SetKey, matches)
    } else if let Some(matches) = matches.subcommand_matches("ls") {
        run(Command::List, matches)
    } else if let Some(matches) = matches.subcommand_matches("add") {
        run(Command::Add, matches)
    } else if let Some(matches) = matches.subcommand_matches("pause") {
        run(Command::Pause, matches)
    } else if let Some(matches) = matches.subcommand_matches("ping") {
        run(Command::Ping, matches)
    } else if let Some(matches) = matches.subcommand_matches("del") {
        run(Command::Delete, matches)
    } else {
        Ok(())
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
