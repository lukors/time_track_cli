#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate clap;
extern crate directories;
extern crate time_track;

use chrono::{
    ParseResult,
    {prelude::*, Duration},
};
use clap::{App, Arg, SubCommand};
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io,
    path::Path,
};

const YMD_FORMAT: &str = "%Y-%m-%d";
const HM_FORMAT: &str = "%H:%M";
const HMS_FORMAT: &str = "%H:%M:%S";
const YMDHM_FORMAT: &str = "%Y-%m-%d %H:%M";

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
                        .help("The tags to associate with the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("time")
                        .long("time")
                        .short("t")
                        .help("The time to put the event at")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("print")
                .about("Prints all information about the event at the given position")
                .arg(
                    Arg::with_name("position")
                        .help("The position of the event to print")
                        .takes_value(true)
                        .required(true),
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
                    Arg::with_name("back")
                        .help("How many days before \"date\" to start listing")
                        .short("b")
                        .long("back")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("start")
                        .help("What date to start from, defaults to today")
                        .short("s")
                        .long("start")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("end")
                        .help("What date to end at")
                        .short("e")
                        .long("end")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("filter")
                        .help("Only log events with the given tags")
                        .short("f")
                        .long("filter")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("verbose")
                        .help("How much information to write out")
                        .short("v")
                        .multiple(true),
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
                    Arg::with_name("no_message")
                        .long("no_message")
                        .help("Removes the message for the event")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("add_tags")
                        .long("add_tags")
                        .short("a")
                        .help("Tags that should be added to the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("rm_tags")
                        .long("rm_tags")
                        .short("r")
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
                        .help("The short name of the tag to remove")
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
    if let Some(matches) = matches.subcommand_matches("print") {
        print_event(matches, &cfg).unwrap();
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
        Some(t) => match parse_datetime(t, &Local::today(), &Local::now().time()) {
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

    let duration_str = hour_string_from_i64(
        event_db
            .get_event_duration(timestamp)
            .expect("Could not retrieve event from database after storing it"),
    );
    let format_str = format!("{} {}", YMD_FORMAT, HM_FORMAT);
    let time_str = Local.timestamp(timestamp, 0).format(&format_str);
    println!(
        "Added event: {} ({}h): {} {:?}",
        time_str, duration_str, description, &tags
    );

    Ok(())
}

fn remove_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
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

fn print_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let event_db = time_track::EventDB::read(path)?;

    let position = match matches.value_of("position") {
        Some(p) => match p.parse::<usize>() {
            Ok(x) => x,
            Err(e) => {
                println!("Could not parse \"position\" value: {}", e);
                return Ok(());
            }
        },
        None => {
            println!("Could not parse \"position\" value");
            return Ok(());
        }
    };

    let log_event = match event_db.get_log_from_pos(position) {
        Some(e) => e,
        None => {
            println!("Could not find an event at the given position");
            return Ok(());
        }
    };

    let time = Local.timestamp(log_event.timestamp, 0).to_rfc2822();

    let tags = log_event
        .event
        .tag_ids
        .iter()
        .map(|i| &*event_db.tags.get(i).unwrap().short_name)
        .collect::<Vec<&str>>()
        .join(", ");

    let duration = match log_event.duration {
        Some(d) => hour_string_from_i64(d),
        None => "-".to_string(),
    };

    fn print_key_value(key: &str, value: &str) {
        println!("{:>15.15}: {}", key, value);
    }

    print_key_value("Time", &time.to_string());
    print_key_value("Duration", &duration);
    print_key_value("Description", &log_event.event.description);
    print_key_value("Tags", &tags);
    print_key_value("Position", &position.to_string());

    Ok(())
}

fn hour_string_from_i64(x: i64) -> String {
    format!("{:.1}", x as f32 / 60. / 60.).to_string()
}

/// Prints out events from the database in different ways.
fn log(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let event_db = time_track::EventDB::read(path)?;

    if matches.is_present("range") || matches.is_present("back") {
        if matches.is_present("start") || matches.is_present("end") {
            println!("Can't use both \"start\" or \"end\" and \"range\" or \"back\" attributes at the same time");
            return Ok(())
        }
    }

    let range = match matches.value_of("range") {
        Some(r) => match r.parse::<i64>() {
            Ok(i) => i,
            Err(e) => {
                println!("Error when parsing \"range\" argument: {:?}", e);
                return Ok(())
            }
        },
        None => 0,
    };

    let back = match matches.value_of("back") {
        Some(b) => match b.parse::<i64>() {
            Ok(d) => d,
            Err(e) => {
                println!("Error when parsing \"back\" argument: {:?}", e);
                return Ok(())
            }
        },
        None => 0,
    };

    let end: chrono::DateTime<Local> = match matches.value_of("end") {
        Some(datetime_str) => match parse_datetime(datetime_str, &Local::today(), &NaiveTime::from_hms(23, 59, 59)) {
            Ok(dt) => dt,
            Err(e) => {
                println!("Error parsing \"end\" argument: {:?}", e);
                return Ok(())
            }
        },
        None => (Local::today() - Duration::days(back)).and_hms(23, 59, 59),
    };

    let start: chrono::DateTime<Local> = match matches.value_of("start") {
        Some(datetime_str) => match parse_datetime(datetime_str, &Local::today(), &NaiveTime::from_hms(00, 00, 00)) {
            Ok(dt) => dt,
            Err(e) => {
                println!("Error parsing \"start\" argument: {:?}", e);
                return Ok(())
            }
        },
        None => (end.date() - Duration::days(range)).and_hms(00, 00, 00),
    };

    let verbosity = match matches.occurrences_of("verbose") {
        0 => 3,
        v => v,
    };

    match verbosity {
        1 => println!("Printing total stats for events between {} and {}",
            start.format(YMDHM_FORMAT), end.format(YMDHM_FORMAT)),
        2 => println!("Printing daily stats for events between {} and {}",
            start.format(YMDHM_FORMAT), end.format(YMDHM_FORMAT)),
        _ => println!("Printing events between {} and {}",
            start.format(YMDHM_FORMAT), end.format(YMDHM_FORMAT)),
    }

    fn print_table(pos: &str, duration: &str, time: &str, tags: &str, description: &str) {
        println!(
            "{:<6.6}|{:<5.5}|{:<6.6}|{:<16.16}|{:<76.76}",
            pos, duration, time, tags, description
        );
    }

    fn print_duration_today(d: i64) {
        println!("Duration: {}", hour_string_from_i64(d));
    }

    let filter_tags = matches.value_of("filter").unwrap_or("");
    let filter_tags: Vec<_> = filter_tags.split_whitespace().collect();
    let filter_tag_ids: Vec<u16> = filter_tags
        .iter()
        .map(|ft| {
            event_db
                .tag_id_from_short_name(ft)
                .expect("Unable to find tag(s) with the given short name(s)")
        })
        .collect();

    if filter_tag_ids.len() > 0 {
        print!("Only including events with the following tags:");
        for tag in filter_tags {
            print!(" {}", tag);
        }
        println!();
    }

    let mut current_date: Option<Date<Local>> = None;

    if verbosity >= 3 {
        print_table("Pos", "Dur", "Time", "Tags", "Description");
    }

    let log_events = event_db.get_log_between_times(
        &start,
        &end,
    );
    let log_events = log_events.iter().filter(|filter_event| {
        filter_tag_ids.iter().all(|filter_tag_id| {
            filter_event
                .event
                .tag_ids
                .iter()
                .any(|tag_id| tag_id == filter_tag_id)
        })
    });

    let mut total_duration = 0i64;
    let mut daily_duration = 0i64;

    for log_event in log_events {
        let event_date = Local.timestamp(log_event.timestamp, 0).date();

        if current_date.is_none() || event_date != current_date.unwrap() {
            if current_date.is_some() {
                if verbosity >= 2 {
                    print_duration_today(daily_duration);
                }
                daily_duration = 0;
            }

            if verbosity >= 2 {
                println!("\n{}", event_date.format("%Y-%m-%d %a"));
            }
            current_date = Some(event_date);
        }

        let duration_string = match log_event.duration {
            Some(d) => {
                if log_event.event.tag_ids.is_empty() {
                    "".to_string()
                } else {
                    total_duration += d;
                    daily_duration += d;
                    hour_string_from_i64(d)
                }
            }
            None => "".to_string(),
        };

        let time_string = Local
            .timestamp(log_event.timestamp, 0)
            .format("%H:%M")
            .to_string();

        let tag_string: String = log_event
            .event
            .tag_ids
            .iter()
            .map(|i| &*event_db.tags.get(i).unwrap().short_name)
            .collect::<Vec<&str>>()
            .join(" ");

        if verbosity >= 3 {
            print_table(
                &log_event.position.to_string(),
                &duration_string,
                &time_string,
                &tag_string,
                &log_event.event.description,
            );
        }
    }

    if verbosity >= 2 {
        print_duration_today(daily_duration);
    }
    println!("\nTotal duration: {}", hour_string_from_i64(total_duration));
    println!("End");

    Ok(())
}

fn parse_datetime(datetime_str: &str, default_date: &Date<Local>, default_time: &NaiveTime) -> ParseResult<DateTime<Local>> {
    match datetime_str {
        "now" => Ok(Local::now()),
        dt_str => {

            Ok(match dt_str.len() {
                5 => {
                    let time = match NaiveTime::parse_from_str(&format!("{}:00", dt_str), HMS_FORMAT) {
                        Ok(r) => r,
                        Err(e) => return Err(e),
                    };
                    default_date.and_hms(time.hour(), time.minute(), time.second())
                }

                10 => {
                    let date = match NaiveDate::parse_from_str(&format!("{}", dt_str), YMD_FORMAT) {
                        Ok(r) => r,
                        Err(e) => return Err(e),
                    };
                    let naive_date_time = date.and_hms(default_time.hour(), default_time.minute(), default_time.second());
                    Local.from_local_datetime(&naive_date_time).unwrap()
                }

                _ => {
                    match Local.datetime_from_str(&dt_str, YMDHM_FORMAT) {
                        Ok(r) => r,
                        Err(e) => return Err(e),
                    }
                }
            })

        }
    }
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
                return Ok(())
            }
        };
    }
    let event_position = event_position;

    if let Some(date_time_str) = matches.value_of("time") {
        let event = match event_db.get_log_from_pos(event_position) {
            Some(r) => r,
            None => {
                println!("Couldn't find an event at the given position: {}", event_position);
                return Ok(())
            }
        };
        let event_time = Local.timestamp(event.timestamp, 0);

        let date_time = match parse_datetime(date_time_str, &event_time.date(), &event_time.time()) {
            Ok(dt) => dt,
            Err(e) => {
                println!("Error parsing date/time: {:?}", e);
                return Ok(())
            }
        };

        let event = match event_db.remove_event(event_position) {
            Some(e) => e,
            None => {
                println!("Could not find an event at the given position: {}", event_position);
                return Ok(())
            }
        };
        event_db.events.insert(date_time.timestamp(), event);
    }

    let no_message = matches.is_present("no_message");

    if let Some(message) = matches.value_of("message") {
        if no_message {
            println!("Can't use both the 'message' and 'no_message' attributes at the same time");
            return Ok(())
        }

        match event_db.get_event_mut(event_position) {
            Some(e) => {
                e.description = message.to_string();
            }
            None => {
                println!("Could not find an event at the given position");
                return Ok(())
            }
        };
    }

    if no_message {
        match event_db.get_event_mut(event_position) {
            Some(e) => e.description = String::new(),
            None => {
                println!("Could not find an event at the given position");
                return Ok(())
            }
        }
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
