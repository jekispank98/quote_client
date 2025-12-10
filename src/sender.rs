use crate::error::ParserError;
use crate::model::command::Command;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

const INTERVAL_MS: u64 = 2000;

pub struct CommandSender {
    udp_socket: UdpSocket,
}

impl CommandSender {
    pub fn new(bind_address: &str) -> Result<Self, ParserError> {
        let socket = UdpSocket::bind(bind_address)?;
        Ok(Self { udp_socket: socket })
    }

    pub fn send_to(
        &self,
        command: &Command,
        target_address: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::encode_to_vec(command, bincode::config::standard())?;
        self.udp_socket.send_to(&encoded, target_address)?;
        Ok(())
    }

    pub fn start_broadcasting(self, target_ip: String, target_port: String, ping: Command) {
        let raw_addr = format!("{}:{}", target_ip.trim(), target_port.trim());

        if let Err(e) = raw_addr.parse::<std::net::SocketAddr>() {
            eprintln!(
                "PING THREAD ERROR: Invalid address format '{}': {}",
                raw_addr, e
            );
            return;
        }
        println!("Ping thread started. Target: {}", raw_addr);

        loop {
            match self.send_to(&ping, &raw_addr) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Ping send error: {:?}", e);
                }
            }

            thread::sleep(Duration::from_millis(INTERVAL_MS));
        }
    }
}
