use chrono::{
    ParseResult,
    {prelude::*, Duration},
};
use clap::{App, Arg, SubCommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    fs::{self, File},
    io,
    path::Path,
};
use terminal_size::{terminal_size, Height, Width};
use time_track::{EventId, TagId};

const DEFAULT_TERMINAL_WIDTH: usize = 100;
const VERSION: &str = env!("CARGO_PKG_VERSION");

const HM_FORMAT: &str = "%H:%M";
const HMS_FORMAT: &str = "%H:%M:%S";
const YMD_FORMAT: &str = "%Y-%m-%d";
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
    let matches = App::new("TimeTrack CLI")
        .version(VERSION)
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
                    Arg::with_name("project")
                        .help("The project to associate with the event")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("time")
                        .long("time")
                        .short("t")
                        .help("The time and/or day to put the event at, the format is hh:mm or 'YYYY-MM-DD hh:mm'")
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
                        .help("Only log events in the given projects")
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
                        .help("The new time and/or day for the event, the format is hh:mm or 'YYYY-MM-DD hh:mm'")
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
                    Arg::with_name("no-message")
                        .long("no-message")
                        .help("Removes the message for the event")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("project")
                        .long("project")
                        .help("Change the event's project")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("no-project")
                        .long("no-project")
                        .help("Remove the project from the event")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("projects")
                .about("Lists all available projects")
        )
        .subcommand(
            SubCommand::with_name("add-project")
                .about("Adds a project to the database")
                .arg(
                    Arg::with_name("short")
                        .short("s")
                        .long("short")
                        .help("The short name for the project that can be quickly written in the terminal")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("long")
                        .short("l")
                        .long("long")
                        .help("The long name for the project for pretty printing")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm-project")
                .about("Removes a project from the database")
                .arg(
                    Arg::with_name("short")
                        .help("The short name of the project to remove")
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
    if let Some(_matches) = matches.subcommand_matches("projects") {
        list_projects(&cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("add-project") {
        add_project(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("rm-project") {
        remove_project(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("config") {
        config(matches, &cfg).unwrap();
    }
}

fn add_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let timestamp = match matches.value_of("time") {
        Some(t) => match parse_datetime(t, Local::today(), Local::now().time()) {
            Ok(dt) => dt.timestamp(),
            Err(e) => {
                println!("Error parsing date/time: {:?}", e);
                return Ok(());
            }
        },
        None => Utc::now().timestamp(),
    };

    let description = matches.value_of("message").unwrap_or("");
    let tag = matches.value_of("project").unwrap_or("");

    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDb::read(path)?;

    if let Some(tag_id) = event_db.tag_id_from_short_name(tag) {
        event_db.add_event(timestamp, description, tag_id).unwrap();
        event_db.write(path)?;
    } else {
        print!(
            "Failed to add event, project with short name does not exist: '{}'",
            tag
        );
        return Ok(());
    }

    let duration_str = hour_string_from_i64(
        event_db
            .get_event_duration(&EventId::Timestamp(timestamp))
            .unwrap_or(0),
    );

    let format_str = format!("{} {}", YMD_FORMAT, HM_FORMAT);
    let time_str = Local.timestamp(timestamp, 0).format(&format_str);
    println!(
        "Added event: {} ({}h): {} {:?}",
        time_str, duration_str, description, &tag
    );

    Ok(())
}

fn remove_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDb::read(path)?;

    let event_id = match matches.value_of("position") {
        Some(position) => match position.parse::<usize>() {
            Ok(p) => EventId::Position(p),
            _ => {
                println!("Could not parse position value");
                return Ok(());
            }
        },
        None => EventId::Position(0),
    };

    match event_db.remove_event(&event_id) {
        Some(e) => {
            event_db.write(path)?;
            println!("Removed {:?}", e);
        }
        None => println!("Could not find an event at the given position"),
    };

    Ok(())
}

fn print_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let event_db = time_track::EventDb::read(path)?;

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

    let log_event = match event_db.get_log(&EventId::Position(position)) {
        Some(e) => e,
        None => {
            println!("Could not find an event at the given position");
            return Ok(());
        }
    };

    let time = Local.timestamp(log_event.timestamp, 0).to_rfc2822();

    let tag = if let Some(tag) = event_db.tag_from_tag_id(log_event.event.tag_id) {
        tag.long_name.clone()
    } else {
        "".to_string()
    };

    // let tag = log_event
    //     .event
    //     .tag_id
    //     .iter()
    //     .map(|i| &*event_db.tags[i].short_name)
    //     .collect::<Vec<&str>>()
    //     .join(", ");

    let duration = match log_event.duration {
        Some(d) => hour_string_from_i64(d),
        None => "-".to_string(),
    };

    fn print_key_value(key: &str, value: &str) {
        println!("{:>15.15}: {}", key, value);
    }

    print_key_value("Time", &time);
    print_key_value("Duration", &duration);
    print_key_value("Message", &log_event.event.description);
    print_key_value("Project", &tag);
    print_key_value("Position", &position.to_string());

    Ok(())
}

fn hour_string_from_i64(x: i64) -> String {
    format!("{:.1}", x as f32 / 60. / 60.)
}

/// Prints out events from the database in different ways.
fn log(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let event_db = time_track::EventDb::read(path)?;

    if (matches.is_present("range") || matches.is_present("back"))
        && (matches.is_present("start") || matches.is_present("end"))
    {
        println!("Can't use both \"start\" or \"end\" and \"range\" or \"back\" attributes at the same time");
        return Ok(());
    }

    let range = match matches.value_of("range") {
        Some(r) => match r.parse::<i64>() {
            Ok(i) => i,
            Err(e) => {
                println!("Error when parsing \"range\" argument: {:?}", e);
                return Ok(());
            }
        },
        None => 0,
    };

    let back = match matches.value_of("back") {
        Some(b) => match b.parse::<i64>() {
            Ok(d) => d,
            Err(e) => {
                println!("Error when parsing \"back\" argument: {:?}", e);
                return Ok(());
            }
        },
        None => 0,
    };

    let end: chrono::DateTime<Local> = match matches.value_of("end") {
        Some(datetime_str) => match parse_datetime(
            datetime_str,
            Local::today(),
            NaiveTime::from_hms(23, 59, 59),
        ) {
            Ok(dt) => dt,
            Err(e) => {
                println!("Error parsing \"end\" argument: {:?}", e);
                return Ok(());
            }
        },
        None => (Local::today() - Duration::days(back)).and_hms(23, 59, 59),
    };

    let start: chrono::DateTime<Local> = match matches.value_of("start") {
        Some(datetime_str) => match parse_datetime(
            datetime_str,
            Local::today(),
            NaiveTime::from_hms(00, 00, 00),
        ) {
            Ok(dt) => dt,
            Err(e) => {
                println!("Error parsing \"start\" argument: {:?}", e);
                return Ok(());
            }
        },
        None => (end.date() - Duration::days(range)).and_hms(00, 00, 00),
    };

    let verbosity = match matches.occurrences_of("verbose") {
        0 => 3,
        v => v,
    };

    // Can the `start.format` and `end.format` calls here be de-duplicated?
    match verbosity {
        1 => println!(
            "Printing total stats for events between {} and {}",
            start.format(YMDHM_FORMAT),
            end.format(YMDHM_FORMAT)
        ),
        2 => println!(
            "Printing daily stats for events between {} and {}",
            start.format(YMDHM_FORMAT),
            end.format(YMDHM_FORMAT)
        ),
        _ => println!(
            "Printing events between {} and {}",
            start.format(YMDHM_FORMAT),
            end.format(YMDHM_FORMAT)
        ),
    }

    fn print_table(pos: &str, duration: &str, time: &str, tags: &str, description: &str) {
        let terminal_width: usize = match terminal_size() {
            Some((Width(w), Height(_))) => w.into(),
            None => DEFAULT_TERMINAL_WIDTH,
        };

        let head = format!(
            "{:<6.6}|{:<5.5}|{:<6.6}|{:<16.16}|",
            pos, duration, time, tags
        );

        let tail_length: usize =
            max(terminal_width as i16 - head.chars().count() as i16 - 1, 4) as usize;

        let output = format!(
            "{}{:<width$.width$}",
            head,
            description,
            width = tail_length
        );

        println!("{}", output.trim());
    }

    fn print_duration_today(d: i64) {
        println!("Duration: {}", hour_string_from_i64(d));
    }

    let filter_tags = matches.value_of("filter").unwrap_or("");
    let filter_tags: Vec<_> = filter_tags.split_whitespace().collect();
    let filter_tag_ids: Vec<TagId> = filter_tags
        .iter()
        .map(|ft| {
            event_db
                .tag_id_from_short_name(ft)
                .expect("Unable to find project(s) with the given short name(s)")
        })
        .collect();

    if !filter_tag_ids.is_empty() {
        print!("Only including events with the following projects:");
        for tag in filter_tags {
            print!(" {}", tag);
        }
        println!();
    }

    let mut current_date: Option<Date<Local>> = None;

    if verbosity >= 3 {
        print_table("Pos", "Dur", "Time", "Project", "Message");
    }

    let log_events = event_db.get_log_between_times(&start, &end);
    let log_events = log_events.iter().filter(|filter_event| {
        filter_tag_ids
            .iter()
            .all(|filter_tag_id| filter_event.event.tag_id == *filter_tag_id)
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
                if log_event.event.tag_id == TagId::NoId {
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

        // let tag_string: String = log_event
        //     .event
        //     .tag_id
        //     .map(|i| &*event_db.tags[i].short_name)
        //     .collect::<Vec<&str>>()
        //     .join(" ");

        let tag_string = if let Some(tag) = event_db.tag_from_tag_id(log_event.event.tag_id) {
            tag.long_name.clone()
        } else {
            "".to_string()
        };

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

fn parse_datetime(
    datetime_str: &str,
    default_date: Date<Local>,
    default_time: NaiveTime,
) -> ParseResult<DateTime<Local>> {
    match datetime_str {
        "now" => Ok(Local::now()),
        dt_str => Ok(match dt_str.len() {
            5 => {
                let time = match NaiveTime::parse_from_str(&format!("{}:00", dt_str), HMS_FORMAT) {
                    Ok(r) => r,
                    Err(e) => return Err(e),
                };
                default_date.and_hms(time.hour(), time.minute(), time.second())
            }

            10 => {
                let date = match NaiveDate::parse_from_str(dt_str, YMD_FORMAT) {
                    Ok(r) => r,
                    Err(e) => return Err(e),
                };
                let naive_date_time = date.and_hms(
                    default_time.hour(),
                    default_time.minute(),
                    default_time.second(),
                );
                Local.from_local_datetime(&naive_date_time).unwrap()
            }

            _ => match Local.datetime_from_str(dt_str, YMDHM_FORMAT) {
                Ok(r) => r,
                Err(e) => return Err(e),
            },
        }),
    }
}

fn edit_event(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDb::read(path)?;

    let event_id = match matches.value_of("position") {
        Some(position) => match position.parse::<usize>() {
            Ok(p) => EventId::Position(p),
            _ => {
                println!("Could not parse position value");
                return Ok(());
            }
        },
        None => EventId::Position(0),
    };

    // By checking if the event_id exists in the databse here we can safely use `unwrap()`
    // in the rest of the code with little risk of triggering a panic.
    if !event_id.exists(&event_db) {
        println!("Couldn't find an event at the given position");
        return Ok(());
    }

    let original_event = event_db.get_event(&event_id).unwrap().clone();

    if let Some(date_time_str) = matches.value_of("time") {
        let event_time = Local.timestamp(event_id.to_timestamp(&event_db).unwrap(), 0);
        let date_time =
            parse_datetime(date_time_str, event_time.date(), event_time.time()).unwrap();
        let event = event_db.remove_event(&event_id).unwrap();
        event_db.events.insert(date_time.timestamp(), event);
    }

    // Message
    let no_message = matches.is_present("no-message");
    if matches.is_present("message") && no_message {
        println!("Can't use both `message` and `no-message` flags");
        return Ok(());
    }

    if let Some(message) = matches.value_of("message") {
        event_db.get_event_mut(&event_id).unwrap().description = message.to_string();
    }

    if no_message {
        event_db.get_event_mut(&event_id).unwrap().description = String::new();
    }

    // Project
    let no_project = matches.is_present("no_project");
    if matches.is_present("project") && no_project {
        println!("Can't use both `project` and `no-project` flags");
        return Ok(());
    }

    if let Some(tag) = matches.value_of("project") {
        if let Some(tag_id) = event_db.tag_id_from_short_name(tag) {
            if event_db.set_event_tag(event_id, tag_id).is_err() {
                println!("Couldn't set the event project");
                return Ok(());
            }
        } else {
            println!("Invalid tag short name: [{}]", tag);
            return Ok(());
        }
    }

    if no_project && event_db.set_event_tag(event_id, TagId::NoId).is_err() {
        println!("Couldn't remove the event project");
        return Ok(());
    }

    let edited_event = event_db.get_event(&event_id);

    event_db.write(path)?;
    println!("Sucessfully edited the event");
    println!("Original: {:?}", original_event);
    println!("  Edited: {:?}", edited_event);
    Ok(())
}

fn list_projects(config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let event_db = time_track::EventDb::read(path)?;

    println!("Projects:");
    for (id, tag) in event_db.tags.iter() {
        println!("{}: {} - {}", id, tag.short_name, tag.long_name);
    }

    Ok(())
}

fn add_project(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDb::read(path)?;

    // I can unwrap these because these arguments are required in Clap.
    let long_name = matches.value_of("long").unwrap();
    let short_name = matches.value_of("short").unwrap();

    let id = match event_db.add_tag(long_name, short_name) {
        Ok(id) => id,
        Err(e) => {
            println!(
                "Could not add project with short name '{short}': {error}",
                short = short_name,
                error = e,
            );
            return Ok(());
        }
    };

    event_db.write(path)?;

    println!(
        "Added project '{long}' (ID: '{id}', short name: '{short}')",
        id = id,
        short = short_name,
        long = long_name,
    );

    Ok(())
}

fn remove_project(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut event_db = time_track::EventDb::read(path)?;

    if let Some(short_name) = matches.value_of("short") {
        if let Some(tag_id) = event_db.tag_id_from_short_name(short_name) {
            event_db.remove_tag(tag_id).unwrap();
            event_db.write(path)?;
        } else {
            println!("Project with short name does not exist: '{}'", short_name);
        }
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
