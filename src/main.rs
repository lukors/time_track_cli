#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate clap;
extern crate time_track;

use chrono::{Duration, prelude::*,};
use clap::{App, Arg, SubCommand};
use std::{fs::File,
          io::{self, prelude::*},
          path::Path};
use time_track::{Event, EventDB};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    path: String,
}
const CONFIG_PATH: &str = "config.json";

impl Config {
    fn read() -> io::Result<Config> {
        // TODO: Handle case where there is no config file.

        let file = File::open(CONFIG_PATH)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }

    fn write(&self) -> io::Result<()> {
        let file = File::create(CONFIG_PATH)?;
        serde_json::to_writer_pretty(&file, self)?;
        Ok(())
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
                        .short("m")
                        .long("message")
                        .help("A description for the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .short("t")
                        .long("tags")
                        .help("The tags to associate with the event")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Removes an event based on its time, or more recent event")
                .arg(
                    Arg::with_name("time")
                        .short("t")
                        .long("time")
                        .help("The UNIX time for the event to remove")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("log")
                .about("Lists events on a given day")
                .arg(
                    Arg::with_name("day")
                        .short("d")
                        .long("day")
                        .help("Which day to list")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Make changes to an event")
                .arg(
                    Arg::with_name("position")
                        .short("p")
                        .long("position")
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
    let time = Utc::now().timestamp();
    let description = matches.value_of("message").unwrap_or("");
    let tags = matches.value_of("tags").unwrap_or("");
    let tags: Vec<_> = tags.split_whitespace().collect();

    let path = Path::new(&config.path);
    let mut event_db = time_track::EventDB::read(path)?;
    event_db.add_event(time, description, &tags).unwrap();
    event_db.write(path)?;

    Ok(())
}

fn remove_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.path);

    match matches.value_of("time") {
        Some(time) => {
            let mut event_db = time_track::EventDB::read(path)?;
            let time = time.parse::<i64>().unwrap();
            match event_db.remove_event(time) {
                Some(event) => {
                    event_db.write(&path)?;
                    println!("Removed event: {:?}", event);
                    return Ok(());
                }
                None => {
                    println!("Could not find an event at that time");
                    return Ok(());
                }
            }
        }
        None => {
            let mut event_db = time_track::EventDB::read(path)?;
            let last_time: i64 = match event_db.events.iter().next_back() {
                Some(event) => *event.0,
                None => return Ok(()),
            };
            match event_db.remove_event(last_time) {
                Some(event) => {
                    event_db.write(&path)?;
                    println!("Removed event:\ntime: {:?} {:?}", last_time, event);
                    return Ok(());
                }
                None => {
                    println!("There are no events to remove");
                    return Ok(());
                }
            }
        }
    }
}

/// Prints out all events on a given day.
fn log(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.path);
    let event_db = time_track::EventDB::read(path)?;

    let input_day = matches.value_of("day");

    let target_day: chrono::Date<Local> = match input_day {
        Some(s) => {
            match s.parse::<i64>() {
                Ok(t) => Local::today() - Duration::days(t),
                Err(_) => {
                    println!("Unrecognized day given");
                    return Ok(());
                },
            }
        },
        None => Local::today(),
    };

    println!(
        "{0: <4} {1: <20} {2: <15} {3: <42}",
        "Pos", "Time", "Tags", "Description"
    );
    for (i, (time, event)) in event_db.events.iter().rev().enumerate() {
        let local_time = Local.timestamp(*time, 0);

        use std::cmp::Ordering;
        match local_time.date().cmp(&target_day) {
            Ordering::Less => break,
            Ordering::Equal => (),
            Ordering::Greater => continue,
        }

        let time_string = local_time
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

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
            "{0: <4} {1: <20} {2: <15} {3: <42}",
            i, time_string, tags_string, description
        );
    }

    Ok(())
}

fn edit_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.path);
    let mut event_db = time_track::EventDB::read(path)?;

    event_db.write(path)?;
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
