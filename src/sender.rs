use crate::error::ParserError;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use crate::model::command::Command;

const INTERVAL_MS: u64 = 5000;
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
        target_address: &String
    ) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::encode_to_vec(command, bincode::config::standard())?;
        self.udp_socket.send_to(&encoded, target_address)?;
        Ok(())
    }

    pub fn start_broadcasting(
        self,
        target_ip: String,
        target_port: String,
        ping: Command
    ) {
        let raw_addr = format!("{}:{}", target_ip.trim(), target_port.trim());

        // Здесь можно обработать ошибку парсинга адреса и выйти из функции, если она критическая
        let target_address = match raw_addr.parse::<std::net::SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("КРИТИЧЕСКАЯ ОШИБКА: Неверный формат адреса '{}': {}", raw_addr, e);
                return; // Выходим из функции, если адрес невалидный
            }
        };

        let target_address_str = target_address.to_string(); // Преобразовать обратно в String для send_to

        loop {
            // ... (Ваш код отправки и обработки ошибок тут)
            match self.send_to(&ping, &target_address_str) {
                Ok(()) => {
                    // ...
                }
                Err(e) => {
                    // Если ошибка I/O возникает здесь, она логируется и цикл продолжается.
                    eprintln!("Sending error (FULL DETAILS): {:?}", e);
                }
            }
            std::thread::sleep(Duration::from_millis(INTERVAL_MS));
        }
    }
}
