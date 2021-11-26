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
use time_track::{CheckpointId, ProjectId};

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
                .about("Adds a new time tracking checkpoint")
                .arg(
                    Arg::with_name("message")
                        .help("A message for the checkpoint")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("project")
                        .help("The project to associate with the checkpoint")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("time")
                        .long("time")
                        .short("t")
                        .help("The time and/or day to put the checkpoint at, the format is hh:mm or 'YYYY-MM-DD hh:mm'")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("print")
                .about("Prints all information about the checkpoint at the given position")
                .arg(
                    Arg::with_name("position")
                        .help("The position of the checkpoint to print")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Removes an checkpoint based on its position")
                .arg(
                    Arg::with_name("position")
                        .help("The position of the checkpoint to remove")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("log")
                .about("Lists checkpoints on a given day")
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
                        .help("Only log checkpoints in the given projects")
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
                .about("Make changes to an checkpoint")
                .arg(
                    Arg::with_name("position")
                        .help("The position in the list of the checkpoint to edit (use log to find position)")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("time")
                        .long("time")
                        .short("t")
                        .help("The new time and/or day for the checkpoint, the format is hh:mm or 'YYYY-MM-DD hh:mm'")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("message")
                        .short("m")
                        .long("message")
                        .help("What the checkpoint's describing message should be changed to")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("no-message")
                        .long("no-message")
                        .help("Removes the message for the checkpoint")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("project")
                        .long("project")
                        .help("Change the checkpoint's project")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("no-project")
                        .long("no-project")
                        .help("Remove the project from the checkpoint")
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
        add_checkpoint(matches, &cfg).expect("Failed adding checkpoint");
    }
    if let Some(matches) = matches.subcommand_matches("rm") {
        remove_checkpoint(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("print") {
        print_checkpoint(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("log") {
        log(matches, &cfg).unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("edit") {
        edit_checkpoint(matches, &cfg).unwrap();
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

fn add_checkpoint(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
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

    let message = matches.value_of("message").unwrap_or("");
    let project = matches.value_of("project").unwrap_or("");
    let mut long_name = String::new();
    let mut no_id = false;

    let path = Path::new(&config.database_path);
    let mut checkpoint_db = time_track::CheckpointDb::read(path)?;

    if let Some(project_id) = checkpoint_db.project_id_from_short_name(project) {
        if let ProjectId::NoId = project_id {
            no_id = true;
        } else if let Some(project) = checkpoint_db.project_from_project_id(project_id) {
            long_name = project.long_name.clone();
        }

        checkpoint_db
            .add_checkpoint(timestamp, message, project_id)
            .unwrap();
        checkpoint_db.write(path)?;
    } else {
        print!(
            "Failed to add checkpoint, project with short name does not exist: '{}'",
            project
        );
        return Ok(());
    }

    let duration_str = hour_string_from_i64(
        checkpoint_db
            .get_checkpoint_duration(&CheckpointId::Timestamp(timestamp))
            .unwrap_or(0),
    );

    let format_str = format!("{} {}", YMD_FORMAT, HM_FORMAT);
    let time_str = Local.timestamp(timestamp, 0).format(&format_str);
    let message = if message.is_empty() {
        "No message".to_string()
    } else {
        format!("'{}'", message)
    };

    if no_id {
        println!(
            "Added empty checkpoint at '{time}' ({duration}h): {message}",
            time = time_str,
            duration = duration_str,
            message = message,
        );
    } else {
        println!(
            "Added checkpoint for '{long}' at '{time}' ({duration}h): {message}",
            time = time_str,
            duration = duration_str,
            message = message,
            long = long_name
        );
    }

    Ok(())
}

fn remove_checkpoint(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut checkpoint_db = time_track::CheckpointDb::read(path)?;

    let checkpoint_id = match matches.value_of("position") {
        Some(position) => match position.parse::<usize>() {
            Ok(p) => CheckpointId::Position(p),
            _ => {
                println!("Could not parse position value");
                return Ok(());
            }
        },
        None => CheckpointId::Position(0),
    };

    match checkpoint_db.remove_checkpoint(&checkpoint_id) {
        Some(e) => {
            checkpoint_db.write(path)?;
            println!("Removed {:?}", e);
        }
        None => println!("Could not find an checkpoint at the given position"),
    };

    Ok(())
}

fn print_checkpoint(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let checkpoint_db = time_track::CheckpointDb::read(path)?;

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

    let log_checkpoint = match checkpoint_db.get_log(&CheckpointId::Position(position)) {
        Some(e) => e,
        None => {
            println!("Could not find an checkpoint at the given position");
            return Ok(());
        }
    };

    let time = Local.timestamp(log_checkpoint.timestamp, 0).to_rfc2822();

    let project = if let Some(project) =
        checkpoint_db.project_from_project_id(log_checkpoint.checkpoint.project_id)
    {
        project.long_name.clone()
    } else {
        "".to_string()
    };

    // let project = log_checkpoint
    //     .checkpoint
    //     .project_id
    //     .iter()
    //     .map(|i| &*checkpoint_db.projects[i].short_name)
    //     .collect::<Vec<&str>>()
    //     .join(", ");

    let duration = match log_checkpoint.duration {
        Some(d) => hour_string_from_i64(d),
        None => "-".to_string(),
    };

    fn print_key_value(key: &str, value: &str) {
        println!("{:>15.15}: {}", key, value);
    }

    print_key_value("Time", &time);
    print_key_value("Duration", &duration);
    print_key_value("Message", &log_checkpoint.checkpoint.message);
    print_key_value("Project", &project);
    print_key_value("Position", &position.to_string());

    Ok(())
}

fn hour_string_from_i64(x: i64) -> String {
    format!("{:.1}", x as f32 / 60. / 60.)
}

/// Prints out checkpoints from the database in different ways.
fn log(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let checkpoint_db = time_track::CheckpointDb::read(path)?;

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
            "Printing total stats for checkpoints between {} and {}",
            start.format(YMDHM_FORMAT),
            end.format(YMDHM_FORMAT)
        ),
        2 => println!(
            "Printing daily stats for checkpoints between {} and {}",
            start.format(YMDHM_FORMAT),
            end.format(YMDHM_FORMAT)
        ),
        _ => println!(
            "Printing checkpoints between {} and {}",
            start.format(YMDHM_FORMAT),
            end.format(YMDHM_FORMAT)
        ),
    }

    fn print_table(pos: &str, duration: &str, time: &str, projects: &str, message: &str) {
        let terminal_width: usize = match terminal_size() {
            Some((Width(w), Height(_))) => w.into(),
            None => DEFAULT_TERMINAL_WIDTH,
        };

        let head = format!(
            "{:<6.6}|{:<5.5}|{:<6.6}|{:<16.16}|",
            pos, duration, time, projects
        );

        let tail_length: usize =
            max(terminal_width as i16 - head.chars().count() as i16 - 1, 4) as usize;

        let output = format!("{}{:<width$.width$}", head, message, width = tail_length);

        println!("{}", output.trim());
    }

    fn print_duration_today(d: i64) {
        println!("Duration: {}", hour_string_from_i64(d));
    }

    let filter_projects = matches.value_of("filter").unwrap_or("");
    let filter_projects: Vec<_> = filter_projects.split_whitespace().collect();
    let filter_project_ids: Vec<ProjectId> = filter_projects
        .iter()
        .map(|ft| {
            checkpoint_db
                .project_id_from_short_name(ft)
                .expect("Unable to find project(s) with the given short name(s)")
        })
        .collect();

    if !filter_project_ids.is_empty() {
        print!("Only including checkpoints with the following projects:");
        for project in filter_projects {
            print!(" {}", project);
        }
        println!();
    }

    let mut current_date: Option<Date<Local>> = None;

    if verbosity >= 3 {
        print_table("Pos", "Dur", "Time", "Project", "Message");
    }

    let log_checkpoints = checkpoint_db.get_log_between_times(&start, &end);
    let log_checkpoints = log_checkpoints.iter().filter(|filter_checkpoint| {
        filter_project_ids
            .iter()
            .all(|filter_project_id| filter_checkpoint.checkpoint.project_id == *filter_project_id)
    });

    let mut total_duration = 0i64;
    let mut daily_duration = 0i64;

    for log_checkpoint in log_checkpoints {
        let checkpoint_date = Local.timestamp(log_checkpoint.timestamp, 0).date();

        if current_date.is_none() || checkpoint_date != current_date.unwrap() {
            if current_date.is_some() {
                if verbosity >= 2 {
                    print_duration_today(daily_duration);
                }
                daily_duration = 0;
            }

            if verbosity >= 2 {
                println!("\n{}", checkpoint_date.format("%Y-%m-%d %a"));
            }
            current_date = Some(checkpoint_date);
        }

        let duration_string = match log_checkpoint.duration {
            Some(d) => {
                if log_checkpoint.checkpoint.project_id == ProjectId::NoId {
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
            .timestamp(log_checkpoint.timestamp, 0)
            .format("%H:%M")
            .to_string();

        // let project_string: String = log_checkpoint
        //     .checkpoint
        //     .project_id
        //     .map(|i| &*checkpoint_db.projects[i].short_name)
        //     .collect::<Vec<&str>>()
        //     .join(" ");

        let project_string = if let Some(project) =
            checkpoint_db.project_from_project_id(log_checkpoint.checkpoint.project_id)
        {
            project.long_name.clone()
        } else {
            "".to_string()
        };

        if verbosity >= 3 {
            print_table(
                &log_checkpoint.position.to_string(),
                &duration_string,
                &time_string,
                &project_string,
                &log_checkpoint.checkpoint.message,
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

fn edit_checkpoint(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut checkpoint_db = time_track::CheckpointDb::read(path)?;

    let checkpoint_id = match matches.value_of("position") {
        Some(position) => match position.parse::<usize>() {
            Ok(p) => CheckpointId::Position(p),
            _ => {
                println!("Could not parse position value");
                return Ok(());
            }
        },
        None => CheckpointId::Position(0),
    };

    // By checking if the checkpoint_id exists in the databse here we can safely use `unwrap()`
    // in the rest of the code with little risk of triggering a panic.
    if !checkpoint_id.exists(&checkpoint_db) {
        println!("Couldn't find an checkpoint at the given position");
        return Ok(());
    }

    let original_checkpoint = checkpoint_db
        .get_checkpoint(&checkpoint_id)
        .unwrap()
        .clone();

    if let Some(date_time_str) = matches.value_of("time") {
        let checkpoint_time =
            Local.timestamp(checkpoint_id.to_timestamp(&checkpoint_db).unwrap(), 0);
        let date_time = parse_datetime(
            date_time_str,
            checkpoint_time.date(),
            checkpoint_time.time(),
        )
        .unwrap();
        let checkpoint = checkpoint_db.remove_checkpoint(&checkpoint_id).unwrap();
        checkpoint_db
            .checkpoints
            .insert(date_time.timestamp(), checkpoint);
    }

    // Message
    let no_message = matches.is_present("no-message");
    if matches.is_present("message") && no_message {
        println!("Can't use both `message` and `no-message` flags");
        return Ok(());
    }

    if let Some(message) = matches.value_of("message") {
        checkpoint_db
            .get_checkpoint_mut(&checkpoint_id)
            .unwrap()
            .message = message.to_string();
    }

    if no_message {
        checkpoint_db
            .get_checkpoint_mut(&checkpoint_id)
            .unwrap()
            .message = String::new();
    }

    // Project
    let no_project = matches.is_present("no_project");
    if matches.is_present("project") && no_project {
        println!("Can't use both `project` and `no-project` flags");
        return Ok(());
    }

    if let Some(project) = matches.value_of("project") {
        if let Some(project_id) = checkpoint_db.project_id_from_short_name(project) {
            if checkpoint_db
                .set_checkpoint_project(checkpoint_id, project_id)
                .is_err()
            {
                println!("Couldn't set the checkpoint project");
                return Ok(());
            }
        } else {
            println!("Invalid project short name: [{}]", project);
            return Ok(());
        }
    }

    if no_project
        && checkpoint_db
            .set_checkpoint_project(checkpoint_id, ProjectId::NoId)
            .is_err()
    {
        println!("Couldn't remove the checkpoint project");
        return Ok(());
    }

    let edited_checkpoint = checkpoint_db.get_checkpoint(&checkpoint_id);

    checkpoint_db.write(path)?;
    println!("Sucessfully edited the checkpoint");
    println!("Original: {:?}", original_checkpoint);
    println!("  Edited: {:?}", edited_checkpoint);
    Ok(())
}

fn list_projects(config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let checkpoint_db = time_track::CheckpointDb::read(path)?;

    println!("Projects:");
    for (id, project) in checkpoint_db.projects.iter() {
        println!("{}: {} - {}", id, project.short_name, project.long_name);
    }

    Ok(())
}

fn add_project(matches: &clap::ArgMatches, config: &Config) -> io::Result<()> {
    let path = Path::new(&config.database_path);
    let mut checkpoint_db = time_track::CheckpointDb::read(path)?;

    // I can unwrap these because these arguments are required in Clap.
    let long_name = matches.value_of("long").unwrap();
    let short_name = matches.value_of("short").unwrap();

    let id = match checkpoint_db.add_project(long_name, short_name) {
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

    checkpoint_db.write(path)?;

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
    let mut checkpoint_db = time_track::CheckpointDb::read(path)?;

    if let Some(short_name) = matches.value_of("short") {
        if let Some(project_id) = checkpoint_db.project_id_from_short_name(short_name) {
            checkpoint_db.remove_project(project_id).unwrap();
            checkpoint_db.write(path)?;
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
