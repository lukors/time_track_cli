#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate clap;
extern crate time_track;

use clap::{App, Arg, SubCommand,};
use std::{fs::File,
		  io::{self, prelude::*},
		  path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
	path: String,
}
const CONFIG_PATH: &str = "config.json";

impl Config {
	fn read() -> io::Result<Config> {
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
		.subcommand(SubCommand::with_name("add")
			.about("Adds a new time tracking event")
		)
		.subcommand(SubCommand::with_name("config")
			.about("Edit the config file")
			.arg(Arg::with_name("path")
				.short("p")
				.long("path")
				.help("Set the path of the database file")
				.value_name("FILE")
				.takes_value(true)
			)
		)
		.get_matches();

	let cfg = Config::read().unwrap();

	if let Some(matches) = matches.subcommand_matches("add") {
		add_event(&cfg);
	}
	if let Some(matches) = matches.subcommand_matches("config") {
		config(matches, &cfg).unwrap();
	}
}

fn add_event(config: &Config) -> io::Result<()> {
	let event_db = time_track::EventDB::read(Path::new(&config.path))?;
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

