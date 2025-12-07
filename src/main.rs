extern crate alloc;
extern crate core;

use crate::args::Args;
use crate::error::ParserError;
use crate::model::command::Command;
use crate::model::tickers::Ticker::C;
use crate::model::tickers::{Ticker, TickerParser};
use crate::sender::CommandSender;
use alloc::fmt::format;
use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::thread::spawn;

mod args;
mod error;
pub mod model;
mod result;
mod sender;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let ip = args.ip.trim().replace("\"", "").to_string();
    let port = args.port.trim().replace("\"", "").to_string();
    let address = format!("{ip}:{port}");
    let file_path = normalize_path(&args.path);
    if is_file_exist(&file_path) {
        println!("Reading file: {:?}", file_path);
        let file = File::open(file_path)
            .map_err(ParserError::Io)
            .expect("Failed to open file");
        let buf = BufReader::new(file);
        let tickers = Ticker::parse_from_file(buf)?;
        println!("tickers: {:?}", tickers);

        let command = Command::new(&ip, &port, tickers);

        println!("Preparing to send to the {ip}:{port}");
        let sender = CommandSender::new("0.0.0.0:0")?;;
        let ip_clone = ip.clone();
        let port_clone = port.clone();
        sender.send_to(&command, &address)?;
        spawn(move || {
            if let Err(e) = start_ping(ip_clone, port_clone) {
                eprintln!("Ошибка в потоке пинга: {}", e);
            }
        });
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
    println!("Check path: {}", path.display());
    let exists = path.exists();
    let is_file = path.is_file();
    println!("exists: {}, is_file: {}", exists, is_file);
    exists && is_file
}

fn start_ping(ip: String, port: String) -> Result<(), Box<dyn std::error::Error>> {
    // Единственная точка отказа - создание сокета
    let sender = CommandSender::new("0.0.0.0:0")?; // Используем ? здесь

    let ping = Command::new_ping(&ip, &port);

    // !!! ИСПРАВЛЕНИЕ: Убираем ? !!!
    sender.start_broadcasting(ip, port, ping);

    // Поскольку start_broadcasting никогда не возвращается,
    // эта строка никогда не будет достигнута.
    // Если поток до нее дойдет (что невозможно), это значит, что цикл сломан.
    Ok(())
}
