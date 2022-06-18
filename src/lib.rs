#[macro_use] extern crate log;

use serde::Deserialize;

pub mod config;
pub mod slack;
pub mod utils;


pub enum SnackError {
    RequestError,
    UnknownService,
    VerificationError,
}

pub struct Snack {
    pub config: config::Configuration,
    pub client: hyper::Client<hyper::client::HttpConnector>
}

pub struct SnackRequest<'a> {
    pub body: &'a [u8],
    pub uri: &'a str,
    pub slack_headers: slack::SlackRequestHeaders<'a>,
}

#[derive(Deserialize)]
pub struct SnackedService {
    pub secret: String,
    pub backend: String,
    pub backend_port: u64,
}

impl Snack {
    pub fn verify_request<'a> (&'a self, request: &SnackRequest) -> Result<&'a SnackedService, SnackError> {
        // Find the service that we're verifying the request for. Snack uses the
        // first part of the URI as the service indicator.
        let (service, _) = utils::parse_resource(request.uri);
    
        // If we don't have a key for this service, error out because we will not
        // be able to validate the headers.
        let service = if let Some(secret) = self.config.services.get(service) {
            secret
        } else {
            return Err(SnackError::UnknownService);
        };
    
        if !slack::verify_signature(&request.slack_headers, &service.secret, request.body) {
            return Err(SnackError::VerificationError);
        }
    
        Ok(service)
    }
}
