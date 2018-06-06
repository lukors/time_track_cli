#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate clap;
extern crate time_track;

use chrono::prelude::*;
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

fn remove_event(matches: &clap::ArgMatches) -> io::Result<()> {
    match matches.value_of("time") {
        Some(time) => {
            println!("some time");
        }
        _ => {
            println!("NOT some time");
        }
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
