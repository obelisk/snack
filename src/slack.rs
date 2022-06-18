use hex;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

use hyper::{Client, Body, Request};
use hyper_tls::HttpsConnector;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value};

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize, Deserialize)]
pub struct SlackConfig {
    token: String,
}

pub fn resign_slack_call(signing_secret: &String, body: &[u8]) -> (String, String) {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    
    let mut mac = HmacSha256::new_varkey(&signing_secret.as_bytes()).expect("Could not calculate HMAC of Slack request");
    // It's always v0 according to the Slack docs
    mac.update(b"v0:");
    mac.update(since_the_epoch.as_secs().to_string().as_bytes());
    mac.update(b":");
    mac.update(body);

    return (since_the_epoch.as_secs().to_string(), hex::encode(mac.finalize().into_bytes()));
}

pub fn verify_signature(signature: &str, timestamp: &str, signing_secret: &str, body: &[u8]) -> bool {
    if signature.len() < 3 {
        return false;
    }
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    
    let itimestamp = timestamp.parse::<u64>().unwrap_or(0);
    let difference = if since_the_epoch.as_secs() > itimestamp {
        since_the_epoch.as_secs() - itimestamp
    } else {
        itimestamp - since_the_epoch.as_secs()
    };

    if difference > 5 {
        return false;
    }

    return calculate_signature(signature, timestamp, signing_secret, body);
}

fn calculate_signature(signature: &str, timestamp: &str, signing_secret: &str, body: &[u8]) -> bool {
    let mut mac = HmacSha256::new_varkey(&signing_secret.as_bytes()).expect("Could not calculate HMAC of Slack request");
    // It's always v0 according to the Slack docs
    mac.update(b"v0:");
    mac.update(timestamp.as_bytes());
    mac.update(b":");
    mac.update(body);
    return hex::encode(mac.finalize().into_bytes()).as_bytes() == &signature.as_bytes()[3..];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_slack_sig_from_api_docs() {
        let signature = "v0=a2114d57b48eac39b9ad189dd8316235a7b4a8d21a10bd27519666489c69b503".to_owned();
        let timestamp = "1531420618".to_owned();
        let signing_secret = "8f742231b10e8888abcd99yyyzzz85a5".to_owned();
        let body = "token=xyzz0WbapA4vBCDEFasx0q6G&team_id=T1DC2JH3J&team_domain=testteamnow&channel_id=G8PSS9T3V&channel_name=foobar&user_id=U2CERLKJA&user_name=roadrunner&command=%2Fwebhook-collect&text=&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2FT1DC2JH3J%2F397700885554%2F96rGlfmibIGlgcZRskXaIFfN&trigger_id=398738663015.47445629121.803a0bc887a14d10d2c447fce8b6703c".to_owned();
        assert!(calculate_signature(&signature, &timestamp, &signing_secret, &body.as_bytes()));
    }

    #[test]
    fn random_bad_sig() {
        let signature = "v0=8750640bace58ac757b8b8f70d2540abfc1b0f673e6f64d3b8a038a5c8c51817".to_owned();
        let timestamp = "1594657912".to_owned();
        let signing_secret = "bbf2d896cedcae67e4574367e95352c2".to_owned();
        let body = "any_random_body".to_owned();
        assert!(!calculate_signature(&signature, &timestamp, &signing_secret, &body.as_bytes()));
    }

    #[test]
    fn random_good_sig() {
        let signature = "v0=23a141a70a89a27cc2c5aa79258167265b90a8bde2faae15bab946fe6ea21d25".to_owned();
        let timestamp = "1594657912".to_owned();
        let signing_secret = "bbf2d896cedcae67e4574367e95352c2".to_owned();
        let body = "any_random_body".to_owned();
        assert!(calculate_signature(&signature, &timestamp, &signing_secret, &body.as_bytes()));
    }

    #[test]
    fn empty_strings() {
        let signature = "".to_owned();
        let timestamp = "".to_owned();
        let signing_secret = "".to_owned();
        let body = "".to_owned();
        assert!(!verify_signature(&signature, &timestamp, &signing_secret, &body.as_bytes()));
    }

    #[test]
    fn random_good_sig_too_old() {
        let signature = "v0=23a141a70a89a27cc2c5aa79258167265b90a8bde2faae15bab946fe6ea21d25".to_owned();
        let timestamp = "1594657912".to_owned();
        let signing_secret = "bbf2d896cedcae67e4574367e95352c2".to_owned();
        let body = "any_random_body".to_owned();
        assert!(!verify_signature(&signature, &timestamp, &signing_secret, &body.as_bytes()));
    }
}