#[macro_use] extern crate log;

extern crate base64;
extern crate hyper;

use snack::config::configure;

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

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!(target: "snack", "Starting Snack - A simple slack command router");
    info!(target: "snack", "Validating Configuration");

    let config = configure().await.unwrap();

    info!(target: "snack", "Providing routing for:");

    for (k, _v) in config.services.iter() {
        info!("\t{}", k);
    }

    let addr = ([0, 0, 0, 0], 7292).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on {}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    info!(target: "slackruster", "Goodbye");
}