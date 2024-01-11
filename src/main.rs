mod udp;

use std::{error::Error, net::UdpSocket};

use deku::DekuContainerWrite;

use crate::udp::Header;

fn main() -> Result<(), Box<dyn Error>> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let response = Header::new().to_bytes()?;
                udp_socket.send_to(&response, source)?;
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
    Ok(())
}
