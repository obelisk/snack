#[macro_use] extern crate log;

use hyper::service::{make_service_fn, service_fn};
use hyper::http::request::Parts;
use hyper::{Client, Body, Request, Response, Server};

pub mod config;
mod slack;
mod utils;

use std::collections::HashMap;

pub enum SnackError {
    RequestError,
    UnknownService,
    VerificationError,
}

pub struct SnackConfig {
    pub services: HashMap<String, String>,
}

pub struct SnackRequest<'a> {
    pub body: &'a [u8],
    pub headers: HashMap<&'a str, &'a str>,
    pub uri: &'a str,
}

pub fn verify_slack_command_request(request: &SnackRequest, config: &SnackConfig) -> Result<(), SnackError> {
    // Find the service that we're verifying the request for. Snack uses the
    // first part of the URI as the service indicator.
    let (service, resource) = utils::parse_resource(request.uri);

    // If we don't have a key for this service, error out because we will not
    // be able to validate the headers.
    let service_secret = if let Some(secret) = config.services.get(service) {
        secret
    } else {
        return Err(SnackError::UnknownService);
    };

    // Extract the two Slack headers we need to validate this request is coming
    // from Slack. This is proved by possesion of a shared HMAC key.
    let (slack_signature, slack_timestamp) = match (request.headers.get("X-Slack-Signature"), request.headers.get("X-Slack-Request-Timestamp")) {
        (Some(signature), Some(timestamp)) => (signature, timestamp),
        _ => return Err(SnackError::RequestError),
    };


    if !slack::verify_signature(slack_signature, slack_timestamp, service_secret, request.body) {
        return Err(SnackError::VerificationError);
    }

    Ok(())
}