use alloc::string::String;
use alloc::vec::Vec;
use bincode::{Decode, Encode};
use clap::command;
use serde::{Deserialize, Serialize};
use crate::model::tickers::Ticker;

const HEADER: &str = "J_QUOTE";
const PING: &str = "PING";
const CONNECTION: &str = "udp";
#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct Command {
    pub header: String,
    pub connection: String,
    pub address: String,
    pub port: String,
    pub tickers: Vec<Ticker>,
}

impl Command {
    pub fn new(address: &str, port: &str, tickers: Vec<Ticker>) -> Self {
        Command {
            header: String::from(HEADER),
            connection: String::from(CONNECTION),
            address: String::from(address),
            port: String::from(port),
            tickers
        }
    }

    pub fn new_ping(ip: &str, port: &str) -> Self {
        Command {
            header: String::from(PING),
            connection: String::from(CONNECTION),
            address: String::from(ip),
            port: String::from(port),
            tickers: Vec::new()
        }
    }
}
