#[macro_use] extern crate log;

pub mod config;
pub mod slack;
pub mod utils;


pub enum SnackError {
    RequestError,
    UnknownService,
    VerificationError,
}


pub struct SnackRequest<'a> {
    pub body: &'a [u8],
    pub uri: &'a str,
    pub slack_headers: slack::SlackRequestHeaders<'a>,
}