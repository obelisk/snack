#[macro_use] extern crate log;

extern crate base64;
extern crate hyper;

use snack::{SnackRequest, slack};
use snack::config::{configure, Configuration};

use std::collections::HashMap;
use std::str;
use std::convert::Infallible;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::http::request::Parts;
use hyper::{Client, Body, Request, Response, Server};
use hyper::{StatusCode};

use serde::{Deserialize, Serialize};

pub fn create_internal_server_error(msg: &str) -> Response<Body> {
    let resp = Response::new("Internal Server Error");
    let (mut parts, _) = resp.into_parts();
    parts.status = StatusCode::INTERNAL_SERVER_ERROR;
    return Response::from_parts(parts, Body::from(msg.to_string()));
}

async fn handle_connection(mut req: Request<Body>, config: Arc<Configuration>) -> Result<Response<Body>, Infallible> {
    let (head, body) = req.into_parts();
    let slack_headers = match slack::extract_slack_headers(&head.headers) {
        Ok(v) => v,
        _ => return Ok(Response::new("Invalid".into()))
    };
    
    let uri =  head.uri.path_and_query().map(|x| x.as_str()).unwrap_or("");

    let body = match hyper::body::to_bytes(body).await {
        Ok(v) => v,
        _ => return Ok(Response::new("Invalid".into())),
    };

    let snack_request = SnackRequest {
        slack_headers,
        body: &body,
        uri,
    };

    match config.verify_request(&snack_request) {
        Ok(_) => Ok(Response::new("Hello, World".into())),
        Err(_) => Ok(Response::new("Invalid".into())),
    }

}

/*
    // Extract the two Slack headers we need to validate this request is coming
    // from Slack. This is proved by possesion of a shared HMAC key.
    let (slack_signature, slack_timestamp) = match (request.headers.get("X-Slack-Signature"), request.headers.get("X-Slack-Request-Timestamp")) {
        (Some(signature), Some(timestamp)) => (signature, timestamp),
        _ => return Err(SnackError::RequestError),
    };

*/

#[tokio::main]
async fn main() {
    env_logger::init();
    info!(target: "snack", "Starting Snack - A simple slack command router");
    info!(target: "snack", "Validating Configuration");

    let config = configure().await.unwrap();
    let config = Arc::new(config);

    info!(target: "snack", "Providing routing for:");

    for (k, _v) in config.services.iter() {
        info!("\t{}", k);
    }

    let addr = ([0, 0, 0, 0], 7292).into();

    let make_svc = make_service_fn(move |_| {
        let config = config.clone();
        async move {
            return Ok::<_, hyper::Error>(service_fn(move |req| 
                handle_connection(req, config.clone())
            ))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on {}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    info!(target: "slackruster", "Goodbye");
}