mod udp;

use std::{error::Error, net::UdpSocket};

use deku::DekuContainerWrite;

use crate::udp::{Dns, QRIndicator};

fn main() -> Result<(), Box<dyn Error>> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_size, source)) => {
                // let mut dns = Dns::try_from(&buf[..size])?;
                let mut dns = Dns::default();

                dns.header.question_count = 1;
                dns.header.qr_indicator = QRIndicator::Response;
                dns.header.answer_record_count = 1;

                let response = dns.to_bytes()?;
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
