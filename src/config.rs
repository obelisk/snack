use clap::{Command, Arg};
use serde::Deserialize;

use std::collections::HashMap;

use crate::{SnackRequest, SnackError, utils, slack};

#[derive(Deserialize)]
pub struct Service {
    pub secret: String,
    pub backend: String,
    pub backend_port: u64,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub services: HashMap<String, Service>,
}

#[derive(Debug)]
pub enum ConfigurationError {
    FileError,
    ParsingError,
}


impl Configuration {
    pub fn verify_request(&self, request: &SnackRequest) -> Result<(), SnackError> {
        // Find the service that we're verifying the request for. Snack uses the
        // first part of the URI as the service indicator.
        let (service, _) = utils::parse_resource(request.uri);
    
        // If we don't have a key for this service, error out because we will not
        // be able to validate the headers.
        let service = if let Some(secret) = self.services.get(service) {
            secret
        } else {
            return Err(SnackError::UnknownService);
        };
    
        if !slack::verify_signature(&request.slack_headers, &service.secret, request.body) {
            return Err(SnackError::VerificationError);
        }
    
        Ok(())
    }
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
    let mut config: Configuration = match toml::from_slice(&config) {
        Ok(config) => config,
        Err(e) => {
            println!("Encountered parsing error while reading configuration!");
            println!("{}", e);
            return Err(ConfigurationError::ParsingError)
        },
    };

    Ok(config)
}