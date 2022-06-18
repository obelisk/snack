use clap::{Command, Arg};
use serde::Deserialize;

use std::collections::HashMap;

use crate::SnackedService;


#[derive(Deserialize)]
pub struct Configuration {
    pub services: HashMap<String, SnackedService>,
}

#[derive(Debug)]
pub enum ConfigurationError {
    FileError,
    ParsingError,
}

pub async fn configure() -> Result<Configuration, ConfigurationError> {
    let matches = Command::new("Snack - A simple Slack command router")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Mitchell Grenier <mitchell@confurious.io>")
        .about("A single Slack ingress location, route to multiple backend services.")
        .arg(
            Arg::new("config")
                .help("Path to the configuration toml file")
                .long("config")
                .default_value("./resources/snack.toml")
                .takes_value(true),
        )
        .get_matches();

    // Read the configuration file
    let config = match tokio::fs::read(matches.value_of("config").unwrap()).await {
        Ok(config) => config,
        Err(e) => {
            println!("Encountered file error when trying to read configuration!");
            println!("{}", e);
            return Err(ConfigurationError::FileError)
        },
    };

    // Parse the TOML into our configuration structures
    let config: Configuration = match toml::from_slice(&config) {
        Ok(config) => config,
        Err(e) => {
            println!("Encountered parsing error while reading configuration!");
            println!("{}", e);
            return Err(ConfigurationError::ParsingError)
        },
    };

    Ok(config)
}