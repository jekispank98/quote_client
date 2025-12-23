//! Quote Client — a UDP client that subscribes to stock quotes from a server and prints
//! received quotes to stdout. It reads a list of tickers from a text file, sends an
//! initial `J_QUOTE` subscription command to the server, keeps the connection alive
//! with periodic `PING`s, and continuously listens for incoming quotes.
//!
//! Usage example (CLI):
//! ```bash
//! quote_client --server-ip 192.168.0.10 --listen-port 55555 --path ./tickers.txt
//! ```
//!
//! The ticker file should contain symbols separated by commas, spaces, or new lines.
//! See `model::tickers` for details.
#![warn(missing_docs)]
mod args;
mod error;
mod model;
mod sender;
mod result;

use crate::args::Args;
use crate::error::ParserError;
use crate::model::command::Command;
use crate::model::quote::Quote;
use crate::model::tickers::{Ticker, TickerParser};
use crate::sender::CommandSender;
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::io::ErrorKind;
use std::net::{TcpStream, UdpSocket};
use std::path::PathBuf;
use result::Result;

/// UDP port on which the quote server is expected to listen.
const SERVER_PORT: &str = "8080";

/// Runs a blocking loop that receives `Quote` messages from the given UDP `socket`
/// and prints them to stdout. Returns an error if receiving or decoding fails.
fn start_receiver_loop(mut socket: TcpStream) -> Result<(), ParserError> {
    println!(
        "Quote receiver running on the port: {}",
        socket.local_addr()?
    );
    let mut buf = [0u8; 1024];

    loop {
        match socket.read(&mut buf) {
            Ok((size)) => {
                match bincode::decode_from_slice::<Quote, _>(
                    &buf[..size],
                    bincode::config::standard(),
                ) {
                    Ok((quote, _)) => {
                        if quote.ticker == "PING" || quote.ticker == "J_QUOTE" { continue }
                        println!(
                            "QUOTE RECEIVED: Ticker={} Price={:.4} Volume={}",
                            quote.ticker, quote.price, quote.volume
                        );
                    }
                    Err(_) => continue,
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::ConnectionReset {
                    continue;
                }
                eprintln!("Receive data error: {}", e);
                return Err(ParserError::Format(format!("{}", e.to_string())));
            }
        }
    }
}

fn main() -> Result<(), ParserError> {
    let args = Args::parse();

    let server_ip = args.server_ip.trim().replace("\"", "").to_string();
    let listen_port = args.listen_port.trim().replace("\"", "").to_string();

    let server_address = format!("{}:{}", server_ip, SERVER_PORT);
    let listen_address = format!("0.0.0.0:{listen_port}");

    let file_path = normalize_path(&args.path);

    if is_file_exist(&file_path) {
        let file = File::open(file_path)
            .map_err(ParserError::Io)
            .expect("Failed to open file");
        let buf = BufReader::new(file);

        let tickers = Ticker::parse_from_file(buf)?;
        println!("Tickers: {:?}", tickers);

        let mut client_socket = TcpStream::connect(&listen_address)?;
        let client_local_addr = client_socket.local_addr()?;

        let command = Command::new(
            &client_local_addr.ip().to_string(),
            &client_local_addr.port().to_string(),
            tickers.clone(),
        );
        println!(
            "Preparing to send J_QUOTE to {} from {}",
            server_address, client_local_addr
        );

        match CommandSender::send_command(&mut client_socket, &command, &server_address) {
            Ok(_) => {
                println!("Initial command sent to server {}.", server_address);
            }
            Err(e) => return Err(ParserError::Format(format!("{}", e.to_string()))),
        };

        let ping_socket = UdpSocket::bind(&server_address)?;
        let ping_command = Command::new_ping(
            &client_local_addr.ip().to_string(),
            &client_local_addr.port().to_string(),
        );

        CommandSender::start_ping_thread(ping_socket, server_address.clone(), ping_command);

        println!("Client is running. Press Ctrl+C to exit.");
        return start_receiver_loop(client_socket);
    }

    Ok(())
}

fn normalize_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    let no_quotes = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed);
    PathBuf::from(no_quotes)
}

fn is_file_exist(path: &PathBuf) -> bool {
    path.exists() && path.is_file()
}
