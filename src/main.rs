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

#[derive(Debug, Deserialize, Serialize)]
struct Config {
	path: String,
}

impl Config {
	fn read() -> io::Result<Config> {
		let file = File::open("config.json")?;
		let config = serde_json::from_reader(file)?;
		Ok(config)
	}

	fn write(&self) -> io::Result<()> {
		let file = File::open("config.json")?;
		let config = serde_json::from_reader(file)?;
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
				.help("Set the path of the database file")
			)
		)
		.get_matches();

		let cfg = Config::read().unwrap();
		if let Some(matches) = matches.subcommand_matches("add") {
			add_event(&cfg);
		}
		if let Some(matches) = matches.subcommand_matches("config") {
			config();
		}
}

fn add_event(config: &Config) -> io::Result<()> {
	let event_db = time_track::EventDB::read(Path::new(&config.path))?;
	Ok(())
}

fn config() {

}

