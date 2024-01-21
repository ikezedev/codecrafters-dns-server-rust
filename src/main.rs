mod udp;

use std::{error::Error, net::UdpSocket};

use deku::{DekuContainerRead, DekuContainerWrite};
use udp::Header;

use crate::udp::{Dns, QRIndicator};

fn main() -> Result<(), Box<dyn Error>> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_size, source)) => {
                let mut header = Header::default();
                let (_, req_header) = Header::from_bytes((&buf, 0))?;

                header.id = req_header.id;
                header.op_code = req_header.op_code;
                header.recursion_desired = req_header.recursion_desired;
                header.response_code = if req_header.op_code == 0 { 0 } else { 4 };

                header.question_count = 1;
                header.qr_indicator = QRIndicator::Response;
                header.answer_record_count = 1;

                let dns = Dns {
                    header,
                    ..Default::default()
                };

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
