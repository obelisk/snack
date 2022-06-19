#[macro_use] extern crate log;

extern crate base64;
extern crate hyper;

use snack::{SnackRequest, slack, Snack, utils};
use snack::config::{configure};

use std::str;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::timeout;

use hyper::service::{make_service_fn, service_fn};
use hyper::{StatusCode, Body, Request, Response, Server, Client};


pub fn create_internal_server_error(msg: &str) -> Response<Body> {
    let resp = Response::new("Internal Server Error");
    let (mut parts, _) = resp.into_parts();
    parts.status = StatusCode::INTERNAL_SERVER_ERROR;
    return Response::from_parts(parts, Body::from(msg.to_string()));
}

async fn handle_connection(req: Request<Body>, snack: Arc<Snack>) -> Result<Response<Body>, Infallible> {
    let (head, body) = req.into_parts();
    let slack_headers = match slack::extract_slack_headers(&head.headers) {
        Ok(v) => v,
        _ => {
            warn!("Request did not have headers");
            return Ok(Response::new("Invalid".into()))
        }
    };
    
    let uri =  head.uri.path_and_query().map(|x| x.as_str()).unwrap_or("");

    let body = match hyper::body::to_bytes(body).await {
        Ok(v) => v,
        _ => {
            error!("Could not fetch body of request");
            return Ok(Response::new("Invalid".into()))
        },
    };

    let snack_request = SnackRequest {
        slack_headers,
        body: &body,
        uri,
    };

    // If we can't verify the request, return invalid and bail.
    // TODO: Change this to a 400.
    let service = match snack.verify_request(&snack_request) {
        Ok(s) => s,
        _ => {
            error!("Could not validate slack request even though we had the correct headers");
            return Ok(Response::new("Invalid".into()))
        },
    };

    let (_, destination_uri) = utils::parse_resource(uri);

    let service_address = format!("http://{}:{}{}", service.backend, service.backend_port, destination_uri);

    let proxy_request_builder = Request::builder()
        .header("X-Snack-Forwarded", "true")
        .uri(service_address)
        .method(&head.method)
        .version(head.version);

    let proxy_request = match proxy_request_builder.body(Body::from(body)) {
        Ok(v) => snack.client.request(v),
        Err(e) => {
            error!("Could not build request to backend: {:?}", e);
            return Ok(Response::new("Invalid".into()))
        },
    };
    
    // If something takes longer than 2500 millis, stop waiting and send Slack an error.
    match timeout(Duration::from_millis(2500), proxy_request).await {
        Ok(Ok(resp)) => Ok(resp),
        Ok(Err(_)) => return Ok(Response::new("Could not process your request. Please contact the service owner".into())),
        Err(_) => return Ok(Response::new("Service took too long to respond. Please contact service owner".into())),
    }

}


#[tokio::main]
async fn main() {
    env_logger::init();
    info!(target: "snack", "Starting Snack - A simple slack command router");
    info!(target: "snack", "Validating Configuration");

    let config = configure().await.unwrap();
    let snack = Snack {
        config,
        client: Client::new(),
    };

    let snack = Arc::new(snack);

    info!(target: "snack", "Providing routing for:");

    for (k, _v) in snack.config.services.iter() {
        info!("\t{}", k);
    }

    let addr = ([0, 0, 0, 0], 7292).into();

    let make_svc = make_service_fn(move |_| {
        let snack = snack.clone();
        async move {
            return Ok::<_, hyper::Error>(service_fn(move |req| 
                handle_connection(req, snack.clone())
            ))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on {}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    info!(target: "snack", "Goodbye");
}
