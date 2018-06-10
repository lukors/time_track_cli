#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate clap;
extern crate time_track;

use chrono::{ParseResult,
             {prelude::*, Duration}};
use clap::{App, Arg, SubCommand};
use std::{fs::File,
          io::{self, prelude::*},
          path::Path,
          str::FromStr};
use time_track::{Event, EventDB};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    path: String,
}
const CONFIG_PATH: &str = "config.json";
const YMD_FORMAT: &str = "%Y-%m-%d";

impl Config {
    fn new() -> Config {
        Config {
            path: "time_track_db.json".to_string(),
        }
    }

    fn read() -> io::Result<Config> {
        let file = File::open(CONFIG_PATH)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }

    fn write(&self) -> io::Result<File> {
        let file = File::create(CONFIG_PATH)?;
        serde_json::to_writer_pretty(&file, self)?;
        Ok(file)
    }
}

fn main() {
    let matches = App::new("Time Track CLI")
        .version("0.1.0")
        .about("Track your time")
        .author("Lukas Orsvärn")
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a new time tracking event")
                .arg(
                    Arg::with_name("message")
                        .help("A description for the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .short("t")
                        .long("tags")
                        .help("The tags to associate with the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("time")
                        .long("time")
                        .help("The time to put the event at")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Removes an event based on its position")
                .arg(
                    Arg::with_name("position")
                        .help("The position of the event to remove")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("log")
                .about("Lists events on a given day")
                .arg(
                    Arg::with_name("day")
                        .help("How many days back from the \"date\" to list events")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("date")
                        .help("What date to start from")
                        .short("d")
                        .long("date")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("range")
                        .help("How many days into the past to list")
                        .short("r")
                        .long("range")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Make changes to an event")
                .arg(
                    Arg::with_name("position")
                        .help("The position in the list of the event to edit (use log to find position)")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("time")
                        .long("time")
                        .short("t")
                        .help("The new time for the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("day")
                        .long("day")
                        .short("d")
                        .help("The new day for the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("message")
                        .short("m")
                        .long("message")
                        .help("What the event's describing message should be changed to")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("add_tags")
                        .long("add_tags")
                        .help("Tags that should be added to the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("rm_tags")
                        .long("rm_tags")
                        .help("Tags that should be removed from the event")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("add_tag")
                .about("Adds a tag to the database")
                .arg(
                    Arg::with_name("short")
                        .short("s")
                        .long("short")
                        .help("The short name for the tag that can be quickly written in the terminal")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("long")
                        .short("l")
                        .long("long")
                        .help("The long name for the tag that for clear printing")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove_tag")
                .about("Removes a tag from the database")
                .arg(
                    Arg::with_name("short")
                        .short("s")
                        .long("short")
                        .help("The short name for the tag to remove")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Edit the config file")
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .long("path")
                        .help("Set the path of the database file")
                        .value_name("FILE")
                        .takes_value(true),
                ),
        )
        .get_matches();

    let cfg = Config::read().unwrap();

    if let Some(matches) = matches.subcommand_matches("add") {
        add_event(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("remove") {
        remove_event(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("log") {
        log(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("edit") {
        edit_event(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("add_tag") {
        add_tag(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("remove_tag") {
        remove_tag(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("config") {
        config(matches, &cfg).unwrap();
    }
}

fn add_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let timestamp = match matches.value_of("time") {
        Some(t) => match parse_datetime(t) {
            Ok(dt) => dt.timestamp(),
            Err(e) => {
                println!("Error parsing date/time: {:?}", e);
                return Ok(());
            }
        },
        None => Utc::now().timestamp(),
    };
    let description = matches.value_of("message").unwrap_or("");
    let tags = matches.value_of("tags").unwrap_or("");
    let tags: Vec<_> = tags.split_whitespace().collect();

    let path = Path::new(&config.path);
    let mut event_db = time_track::EventDB::read(path)?;
    event_db.add_event(timestamp, description, &tags).unwrap();
    event_db.write(path)?;

    Ok(())
}

fn remove_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    // TODO: Edit this function to use the position system instead of specific times.
    let path = Path::new(&config.path);
    let mut event_db = time_track::EventDB::read(path)?;

    let mut event_position = 0;
    if let Some(position) = matches.value_of("position") {
        event_position = match position.parse::<i64>() {
            Ok(p) => p as usize,
            _ => {
                println!("Could not parse position value");
                return Ok(());
            }
        };
    }
    let event_position = event_position;

    match event_db.remove_event(event_position) {
        Some(e) => {
            event_db.write(&path)?;
            println!("Removed {:?}", e);
        }
        None => println!("Could not find an event at the given position"),
    };

    Ok(())
}

/// Prints out all events on a given day.
fn log(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.path);
    let event_db = time_track::EventDB::read(path)?;

    let mut date: chrono::Date<Local> = match matches.value_of("date") {
        Some(date_str) => match parse_date(date_str) {
            Ok(dt) => dt,
            Err(e) => {
                println!("Error parsing date: {:?}", e);
                return Ok(());
            }
        },
        None => Local::today(),
    };

    if let Some(day) = matches.value_of("day") {
        match day.parse::<i64>() {
            Ok(d) => date = date - Duration::days(d),
            Err(e) => {
                println!("Error when parsing \"day\" argument: {:?}", e);
                return Ok(());
            }
        }
    }

    let range = match matches.value_of("range") {
        Some(r) => match r.parse::<i64>() {
            Ok(i) => i,
            Err(e) => {
                println!("Could not parse \"range\": {:?}", e);
                return Ok(())
            },
        },
        None => 0,
    };

    println!(
        "{:<14.14} {:>6.6} {:<15.15} {:<42.42}",
        "Time", "Pos", "Tags", "Description"
    );
    for days_back in 0..range {
        println!("{}", (date - Duration::days(days_back)).format("%Y-%m-%d %a"));
        for (i, (time, event)) in event_db.events.iter().rev().enumerate() {
            let local_time = Local.timestamp(*time, 0);

            use std::cmp::Ordering;
            match local_time.date().cmp(&(date - Duration::days(days_back))) {
                Ordering::Less => break,
                Ordering::Equal => (),
                Ordering::Greater => continue,
            }

            let time_string = local_time.format("%H:%M").to_string();

            let num_tags = event.tag_ids.len();
            let only_tags = event
                .tag_ids
                .iter()
                .map(|i| &*event_db.tags.get(i).unwrap().short_name)
                .collect::<Vec<&str>>()
                .join(" ");
            let tags_string = format!("{}: {}", num_tags, only_tags);

            let description = &event.description;

            println!(
                "{:>14.14} {:>6.6} {: <15.15} {: <42.42}",
                time_string, i, tags_string, description
            );
        }
    }

    Ok(())
}

fn parse_datetime(datetime_str: &str) -> ParseResult<DateTime<Local>> {
    let datetime_str = match datetime_str.contains(' ') {
        true => datetime_str.to_string(),
        false => format!("{} {}", Local::today().format(YMD_FORMAT), datetime_str),
    };

    Local.datetime_from_str(&datetime_str, &format!("{} {}", YMD_FORMAT, "%H:%M"))
}

fn parse_date(date_str: &str) -> ParseResult<Date<Local>> {
    let datetime_str = format!("{} 00:00", date_str);
    Ok(Local
        .datetime_from_str(&datetime_str, &format!("{} {}", YMD_FORMAT, "%H:%M"))?
        .date())
}

fn edit_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.path);
    let mut event_db = time_track::EventDB::read(path)?;

    let mut event_position = 0;
    if let Some(position) = matches.value_of("position") {
        event_position = match position.parse::<i64>() {
            Ok(p) => p as usize,
            _ => {
                println!("Could not parse position value");
                return Ok(());
            }
        };
    }
    let event_position = event_position;

    if let Some(date_time_str) = matches.value_of("time") {
        let date_time = match parse_datetime(date_time_str) {
            Ok(dt) => dt,
            Err(e) => {
                println!("Error parsing date/time: {:?}", e);
                return Ok(());
            }
        };

        let event = match event_db.remove_event(event_position) {
            Some(e) => e,
            None => {
                println!("Could not find an event at the given position");
                return Ok(());
            }
        };
        event_db.events.insert(date_time.timestamp(), event);
    }

    let day = matches.value_of("day");

    if let Some(message) = matches.value_of("message") {
        match event_db.get_event_mut(event_position) {
            Some(e) => {
                e.description = message.to_string();
            }
            None => {
                println!("Could not find an event at the given position");
                return Ok(());
            }
        };
    }

    if let Some(add_tags) = matches.value_of("add_tags") {
        let short_names: Vec<&str> = add_tags.split_whitespace().collect();

        match event_db.add_tags_for_event(event_position, &short_names) {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                return Ok(());
            }
        }
    }

    if let Some(rm_tags) = matches.value_of("rm_tags") {
        let short_names: Vec<&str> = rm_tags.split_whitespace().collect();

        match event_db.remove_tags_for_event(event_position, &short_names) {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                return Ok(());
            }
        }
    }

    event_db.write(path)?;
    println!("Sucessfully edited the event");
    Ok(())
}

fn add_tag(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.path);
    let mut event_db = time_track::EventDB::read(path)?;

    // I can unwrap these because these arguments are required in Clap.
    let long_name = matches.value_of("long").unwrap();
    let short_name = matches.value_of("short").unwrap();

    event_db.add_tag(long_name, short_name).unwrap();
    event_db.write(path)?;

    Ok(())
}

fn remove_tag(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.path);
    let mut event_db = time_track::EventDB::read(path)?;

    if let Some(short_name) = matches.value_of("short") {
        event_db.remove_tag(short_name.to_string());
        event_db.write(path)?;
    }

    Ok(())
}

fn config(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let mut config_new = config.clone();

    if let Some(path) = matches.value_of("path") {
        config_new.path = path.to_string();
    }

    config_new.write()?;

    Ok(())
}
