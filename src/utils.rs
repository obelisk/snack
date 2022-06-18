use std::collections::HashMap;
use std::env;
use std::str;

use hyper::{Body, Client, Request, Response, StatusCode};

use serde::{Deserialize, Serialize};


// Quick and dirty parsing of post parameters
pub fn parse_post_params(body: &str) -> HashMap<&str, &str> {
    let raw_fields = body.split("&");
    let mut fields = HashMap::new();

    for field in raw_fields {
        let parts: Vec<&str> = field.split("=").collect();
        match parts.len() {
            2 => {
                trace!(target: "slackruster", "Post Param: {} : {}", parts[0], parts[1]);
                fields.insert(parts[0], parts[1]);
            },
            _ => debug!(target: "slackruster", "\"{}\" doesn't have exactly two parts, not processing", field),
        }

    }
    return fields;
}

pub fn parse_resource(full_uri: &str) -> (&str, &str) {
    let mut x = 0;
    let mut start = 0;
    for chr in full_uri.chars() {
        match (chr, x) {
            ('/', 0) => {start += 1; x += 1},
            ('/', _) => break,
            _ => x += 1,
        };
    }

    return (&full_uri[start..x], &full_uri[x..]);
}

pub fn create_internal_server_error(msg: &str) -> Response<Body> {
    let resp = Response::new("Internal Server Error");
    let (mut parts, _) = resp.into_parts();
    parts.status = StatusCode::INTERNAL_SERVER_ERROR;
    return Response::from_parts(parts, Body::from(msg.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_params() {
        let body = format!("text=test+message");
        let params = parse_post_params(&body);

        assert!(params.len() == 1);
        assert_eq!(params["text"], "test+message");
    }

    #[test]
    fn parse_more_params() {
        let body = format!("text=test+message&user=UTM1FG7D5");
        let params = parse_post_params(&body);

        assert!(params.len() == 2);
        assert_eq!(params["text"], "test+message");
        assert_eq!(params["user"], "UTM1FG7D5");
    }

    #[test]
    fn parse_slack_like_params() {
        let body = format!("token=BeeFc4bdeaddead0p0kyDKNK&team_id=TTJZZZTFE&team_domain=exampleteam&channel_id=CTXXXYYZZ&channel_name=general&user_id=UTM1FG7D5&user_name=mitchell&command=%2Fping&text=wobbly+short+cake&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2FZZJMKCTFE%2F1231600008902%2Fmy7FkqeYc2xfHOMBBuZB5deh&trigger_id=1244619999999.936739999999.f7fa19b14ddd5e2e4b16da9999999999");
        let params = parse_post_params(&body);

        assert!(params.len() == 11);
        assert_eq!(params["token"], "BeeFc4bdeaddead0p0kyDKNK");
        assert_eq!(params["team_id"], "TTJZZZTFE");
        assert_eq!(params["team_domain"], "exampleteam");
        assert_eq!(params["channel_id"], "CTXXXYYZZ");
        assert_eq!(params["channel_name"], "general");
        assert_eq!(params["user_id"], "UTM1FG7D5");
        assert_eq!(params["user_name"], "mitchell");
        assert_eq!(params["command"], "%2Fping");
        assert_eq!(params["text"], "wobbly+short+cake");
        assert_eq!(params["response_url"], "https%3A%2F%2Fhooks.slack.com%2Fcommands%2FZZJMKCTFE%2F1231600008902%2Fmy7FkqeYc2xfHOMBBuZB5deh");
        assert_eq!(params["trigger_id"], "1244619999999.936739999999.f7fa19b14ddd5e2e4b16da9999999999");
    }


    #[test]
    fn extract_service_empty() {
        let uri = format!("");
        
        assert_eq!(parse_resource(&uri), ("", ""));
    }

    #[test]
    fn extract_service_easy() {
        let uri = format!("/service1/call_site_one");
        
        assert_eq!(parse_resource(&uri), ("service1", "/call_site_one"));
    }

    #[test]
    fn extract_service_no_lead() {
        let uri = format!("service1/call_site_one");
        
        assert_eq!(parse_resource(&uri), ("service1", "/call_site_one"));
    }

    #[test]
    fn extract_service_with_params() {
        let uri = format!("/service1/call_site_one?someother=text&again=banana");
        
        assert_eq!(parse_resource(&uri), ("service1", "/call_site_one?someother=text&again=banana"));
    }

    #[test]
    fn extract_service_single_letter() {
        let uri = format!("/s/call_site_one?someother=text&again=banana");
        
        assert_eq!(parse_resource(&uri), ("s", "/call_site_one?someother=text&again=banana"));
    }


    #[test]
    fn extract_service_malicious_one() {
        let uri = format!("//call_site_one?someother=text&again=banana");
        
        assert_eq!(parse_resource(&uri), ("", "/call_site_one?someother=text&again=banana"));
    }

    #[test]
    fn extract_service_malicious_two() {
        let uri = format!("//////////call_site_one?someother=text&again=banana");

        assert_eq!(parse_resource(&uri), ("", "/////////call_site_one?someother=text&again=banana"));
    }

    #[test]
    fn extract_service_no_service() {
        let uri = format!("call_site_one?someother=text&again=banana");

        assert_eq!(parse_resource(&uri), ("call_site_one?someother=text&again=banana", ""));
    }

    #[test]
    fn extract_service_no_service_ts() {
        let uri = format!("call_site_one?someother=text&again=banana/");

        assert_eq!(parse_resource(&uri), ("call_site_one?someother=text&again=banana", "/"));
    }
}