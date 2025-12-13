//! Sending commands to the quote server over UDP.
//!
//! This module provides a small helper for encoding and sending `Command` messages
//! and for running a background PING loop to keep the subscription alive.
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::io::ErrorKind;
use crate::error::ParserError;
use crate::model::command::Command;

/// PING interval in milliseconds used by the background thread.
const INTERVAL_MS: u64 = 2000;

/// Helper type for sending commands to the server.
pub struct CommandSender;

impl CommandSender {
    /// Encodes the provided `command` using `bincode` and sends it to `target_address`
    /// via the provided UDP `socket`.
    pub fn send_command(
        socket: &UdpSocket,
        command: &Command,
        target_address: &str,
    ) -> Result<(), ParserError> {
        let encoded = bincode::encode_to_vec(command, bincode::config::standard())?;
        socket.send_to(&encoded, target_address)?;
        Ok(())
    }
    
    /// Starts a background thread that periodically sends a `PING` command to `target_addr`.
    ///
    /// The thread sleeps for `INTERVAL_MS` between sending and ignores transient
    /// `ConnectionReset` errors that may occur on Windows when the remote is not yet ready.
    pub fn start_ping_thread(socket: UdpSocket, target_addr: String, ping_command: Command) {
        println!("Ping thread started. Target: {}", target_addr);
        thread::spawn(move || {
            let interval = Duration::from_millis(INTERVAL_MS);
            loop {
                thread::sleep(interval);
                let encoded = match bincode::encode_to_vec(&ping_command, bincode::config::standard()) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("PING THREAD ERROR: Failed to encode PING: {:?}", e);
                        continue;
                    }
                };

                match socket.send_to(&encoded, &target_addr) {
                    Ok(_) => {},
                    Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                        continue;
                    }
                    Err(e) => {
                        eprintln!("PING THREAD ERROR: Failed to send PING to {}: {:?}", target_addr, e);
                    }
                }
            }
        });
    }
}