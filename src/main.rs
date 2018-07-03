#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate clap;
extern crate directories;
extern crate time_track;

use chrono::{
    ParseResult, {prelude::*, Duration},
};
use clap::{App, Arg, SubCommand};
use directories::ProjectDirs;
use std::{
    fs::{self, File}, io, path::Path,
};

const YMD_FORMAT: &str = "%Y-%m-%d";

#[cfg(debug_assertions)]
const CONFIG_FILENAME: &str = "config_debug.json";
#[cfg(not(debug_assertions))]
const CONFIG_FILENAME: &str = "config.json";

#[cfg(debug_assertions)]
const DATABASE_FILENAME: &str = "database_debug.json";
#[cfg(not(debug_assertions))]
const DATABASE_FILENAME: &str = "database.json";

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "Orsvarn";
const APPLICATION: &str = "TimeTrack";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    database_path: String,
}

impl Config {
    fn read() -> io::Result<Config> {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION);

        let config_dir = proj_dirs.config_dir();
        let data_dir = proj_dirs.data_dir();

        let config_path = config_dir.join(CONFIG_FILENAME);
        let database_path = data_dir.join(DATABASE_FILENAME);

        let config;
        if config_path.is_file() {
            let file = File::open(config_path)?;
            config = serde_json::from_reader(file)?;
        } else {
            config = Config {
                database_path: database_path
                    .to_str()
                    .expect("Could not parse database path to string")
                    .to_string(),
            };
            config.write()?;
        }
        Ok(config)
    }

    fn write(&self) -> io::Result<File> {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION);

        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let config_path = config_dir.join(CONFIG_FILENAME);
        let file = File::create(config_path)?;
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
            SubCommand::with_name("rm")
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
                    Arg::with_name("range")
                        .help("How many days into the past to list")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("date")
                        .help("What date to start from, defaults to today")
                        .short("d")
                        .long("date")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("back")
                        .help("How many days before \"date\" to start listing")
                        .short("b")
                        .long("back")
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
            SubCommand::with_name("tags")
                .about("Lists all available tags")
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

    let cfg = Config::read().expect("Could not read config file");

    if let Some(matches) = matches.subcommand_matches("add") {
        add_event(matches, &cfg).expect("Failed adding event");
    }
    if let Some(matches) = matches.subcommand_matches("rm") {
        remove_event(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("log") {
        log(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("edit") {
        edit_event(matches, &cfg).unwrap();
    }
    if let Some(_matches) = matches.subcommand_matches("tags") {
        list_tags(&cfg).unwrap();
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

    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDB::read(path)?;
    event_db.add_event(timestamp, description, &tags).unwrap();
    event_db.write(path)?;

    Ok(())
}

fn remove_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    // TODO: Edit this function to use the position system instead of specific times.
    let path = Path::new(&config.database_path);
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
    let path = Path::new(&config.database_path);
    let event_db = time_track::EventDB::read(path)?;

    let range = match matches.value_of("range") {
        Some(r) => match r.parse::<i64>() {
            Ok(i) => i + 1,
            Err(e) => {
                println!("Could not parse \"range\": {:?}", e);
                return Ok(());
            }
        },
        None => 1,
    };

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

    if let Some(back) = matches.value_of("back") {
        match back.parse::<i64>() {
            Ok(d) => date = date - Duration::days(d),
            Err(e) => {
                println!("Error when parsing \"back\" argument: {:?}", e);
                return Ok(());
            }
        }
    }

    println!(
        "Printing events between {} and {}",
        date.format("%a %Y-%m-%d"),
        (date - Duration::days(range)).format("%a %Y-%m-%d")
    );

    fn print_table(pos: &str, duration: &str, time: &str, tags: &str, description: &str) {
        println!(
            "{:<6.6} {:<4.4} {:<5.5} {:<15.15} {:<46.46}",
            pos, duration, time, tags, description
        );
    }

    print_table("Pos", "Dur", "Time", "Tags", "Description");
    for days_back in 0..range {
        let current_day = date - Duration::days(days_back);

        let mut printed_date = false;

        for (i, (time, event)) in event_db.events.iter().rev().enumerate() {
            let local_time = Local.timestamp(*time, 0);

            use std::cmp::Ordering;
            match local_time.date().cmp(&current_day) {
                Ordering::Less => break,
                Ordering::Equal => (),
                Ordering::Greater => continue,
            }

            if !printed_date {
                print!("{}", current_day.format("%Y-%m-%d %a"));
                if current_day == Local::today() {
                    print!(" (today)");
                }
                println!("");
                printed_date = true;
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

            let duration = {
                if description.is_empty() && num_tags == 0 {
                    "".to_string()
                } else {
                    let duration = event_db.get_event_duration(i).unwrap_or(0);
                    let duration = duration as f32 / 60. / 60.;
                    let duration = format!("{:.1}", duration);
                    duration
                }
            };

            print_table(&i.to_string(), &duration, &time_string, &tags_string, description);
        }
    }

    println!("End");

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
    let path = Path::new(&config.database_path);
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

fn list_tags(config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let event_db = time_track::EventDB::read(path)?;

    println!("Tags:");
    for (id, tag) in event_db.tags_iter() {
        println!("{}: {} - {}", id, tag.short_name, tag.long_name);
    }

    Ok(())
}

fn add_tag(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDB::read(path)?;

    // I can unwrap these because these arguments are required in Clap.
    let long_name = matches.value_of("long").unwrap();
    let short_name = matches.value_of("short").unwrap();

    event_db.add_tag(long_name, short_name).unwrap();
    event_db.write(path)?;

    Ok(())
}

fn remove_tag(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDB::read(path)?;

    if let Some(short_name) = matches.value_of("short") {
        event_db.remove_tag(short_name.to_string()).unwrap();
        event_db.write(path)?;
    }

    Ok(())
}

fn config(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let mut config_new = config.clone();

    if let Some(path) = matches.value_of("path") {
        config_new.database_path = path.to_string();
    }

    config_new.write()?;

    Ok(())
}
