use std::env;
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::{self, IsTerminal};
use clap::{Parser, Subcommand};
use colored::*;

mod api;
use crate::api::ApiClient;

#[cfg(test)]
mod tests;

/// healthchecks.io command line client
#[derive(Parser, Debug)]
#[command(name = "hchk", version = "0.1.0")]
struct Cli {
    /// Be verbose
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Save API key to $HOME/.hchk
    Setkey {
        /// API key
        key: Option<String>,
    },
    /// List checks
    Ls {
        /// Long listing
        #[arg(short = 'l')]
        long: bool,
        /// List 'up' only checks
        #[arg(short = 'u')]
        up: bool,
        /// List 'down' only checks
        #[arg(short = 'd')]
        down: bool,
        /// Filter by name/id
        query: Option<String>,
    },
    /// Add check
    Add {
        /// Name
        name: String,
        /// Schedule in cron format
        schedule: String,
        /// Grace in hours
        grace: Option<String>,
        /// Timezone
        tz: Option<String>,
        /// Tags
        tags: Option<String>,
    },
    /// Delete check
    Del {
        /// Check's ID to delete
        id: String,
    },
    /// Pause check
    Pause {
        /// Check's ID to pause
        id: String,
    },
    /// Ping check
    Ping {
        /// Check's ID to ping
        id: String,
    },
}

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

    let tty = io::stdout().is_terminal();
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
                 id=c.short_uuid,
                 last_ping=c.humanized_last_ping_at(),
                 s_width=6, id_width=9, n_width=40, lp_width=30);
    }

    Ok(())
}

fn cmd_add_check(client: &ApiClient, name: &str, schedule: &str, grace: Option<&str>, tz: Option<&str>, tags: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let grace_s = grace.unwrap_or("1");
    let grace_v = grace_s.parse::<u32>()
        .map_err(|_| format!("Grace period must be a valid number, got: {}", grace_s))?;

    let check = client.add(name, schedule, grace_v, tz, tags)?;
    println!("{} {} {}", check.name, check.uuid, check.ping_url);

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

const API_KEY_ENV: &str = "HCHK_API_KEY";
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

fn run(cmd: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    let key = match cmd {
        Commands::Setkey { .. } => "".to_string(),
        _ => get_api_key()?
    };

    let client = ApiClient::new(&key, None);

    match cmd {
        Commands::Ls { long, up, down, query } => {
            let flags = LsFlags {
                long: *long,
                up: *up,
                down: *down,
            };
            cmd_list_checks(&client, &flags, query.as_deref())
        }
        Commands::Add { name, schedule, grace, tz, tags } => {
            cmd_add_check(
                &client,
                name,
                schedule,
                grace.as_deref(),
                tz.as_deref(),
                tags.as_deref(),
            )
        }
        Commands::Ping { id } => cmd_ping_check(&client, Some(id)),
        Commands::Pause { id } => cmd_pause_check(&client, Some(id)),
        Commands::Del { id } => cmd_delete_check(&client, Some(id)),
        Commands::Setkey { key } => cmd_setkey(key.as_deref()),
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle the subcommand if present
    let result = if let Some(command) = &cli.command {
        run(command)
    } else {
        Ok(())
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
