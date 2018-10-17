extern crate simple_error;
extern crate chrono;
extern crate chrono_tz;
extern crate chrono_humanize;
extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

// requires feature: `ureq = { version = "*", features = ["json"] }`
extern crate ureq;

extern crate clap;
extern crate colored;
extern crate isatty;

use std::env;
use std::process;
use clap::{Arg, App, SubCommand};
use colored::*;
use isatty::{stdout_isatty};

mod api;

fn colored_status(status: &String) -> ColoredString {
    let c = match status.as_ref() {
        "up" => "green",
        "down" => "red",
        "grace" => "cyan",
        "paused" => "yellow",
        _ => "white",
    };

    return status.color(c);
}

fn cmd_list_checks(api_key: &str, query: Option<&str>) {
    let re = api::get_checks(api_key, query);
    if re.is_err() {
        println!("err {:?}", re);
        return
    }

    let mut checks = re.unwrap();
    println!("total {:?}", checks.len());

    checks.sort_by(|a, b| a.name.cmp(&b.name));

    for c in checks {
        let mut status = colored_status(&c.status);
        if !stdout_isatty() {
            status = status.clear();
        }

        println!("{status:<s_width$} {id:<id_width$} {name:<n_width$} {last_ping:<lp_width$}",
                 name=c.name,
                 status=status,
                 id=c.short_id(),
                 last_ping=c.humanized_last_ping_at(),
                 s_width=6, id_width=9, n_width=40, lp_width=30);
    }
}

fn cmd_add_check(api_key: &str, name: &str, schedule: &str) {
    let re = api::find_check(api_key, name);
    if re.is_none() {
        return
    }

    let c = re.unwrap();
    let re = api::delete_check(api_key, &c);
    if re.is_err() {
        println!("err {:?}", re);
        return
    }

    //let mut check = re.unwrap();
    //println!("check {}", check.status)
}


fn cmd_pause_check(api_key: &str, id: Option<&str>) {
    let re = api::find_check(api_key, id.unwrap());
    if re.is_none() {
        return
    }

    let c = re.unwrap();
    if c.status == "paused" {
        println!("{}: check is already paused", c.name);
        return
    }

    let re = api::pause_check(api_key, &c);
    if re.is_err() {
        println!("err {:?}", re);
        return
    }
}

fn cmd_ping_check(api_key: &str, id: Option<&str>) {
    let re = api::find_check(api_key, id.unwrap());
    if re.is_none() {
        return
    }

    let c = re.unwrap();
    let re = api::ping_check(api_key, &c);
    if re.is_err() {
        println!("err {:?}", re);
        return
    }
}

fn cmd_delete_check(api_key: &str, id: Option<&str>) {
    let re = api::find_check(api_key, id.unwrap());
    if re.is_none() {
        return
    }

    let c = re.unwrap();
    let re = api::delete_check(api_key, &c);
    if re.is_err() {
        println!("err {:?}", re);
        return
    }

    //let mut check = re.unwrap();
    //println!("check {}", check.status)
}

const API_KEY_ENV: &'static str = "HCHK_API_KEY";
fn get_api_key() -> String {
    let key = env::var(API_KEY_ENV);

    if key.is_err() {
        println!("please set {} environment variable", API_KEY_ENV);
        process::exit(1);
    }
    //println!("api_key {:?}", get_api_key());
    return key.unwrap()
}

enum Command {
    Add,
    Delete,
    Pause,
    Ping,
    List,
}

fn run(cmd: Command, args: &clap::ArgMatches) {
    let skey = get_api_key();
    let key = skey.as_str();

    match cmd {
        Command::List => cmd_list_checks(key, args.value_of("query")),
        Command::Ping => cmd_ping_check(key, args.value_of("id")),
        Command::Pause => cmd_pause_check(key, args.value_of("id")),
        Command::Delete => cmd_delete_check(key, args.value_of("id")),
        _ => println!("not implemented yet"),
    }
}

fn main() {
    let matches = App::new("hchk")
        .version("1.0")
        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("be verbose"))
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
                    .arg(Arg::with_name("schedule").help("schedule in cron format").required(true)))

        .get_matches();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(matches) = matches.subcommand_matches("ls") {
        //println!("{:?}", matches);
        run(Command::List, matches)
    } else if let Some(matches) = matches.subcommand_matches("pause") {
        run(Command::Pause, matches)
    } else if let Some(matches) = matches.subcommand_matches("ping") {
        run(Command::Ping, matches)
    } else if let Some(matches) = matches.subcommand_matches("del") {
        run(Command::Delete, matches)
    }
    //list_checks()
}
