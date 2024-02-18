mod traits;
mod udp;

use std::{error::Error, net::UdpSocket};

use deku::{DekuContainerRead, DekuContainerWrite};
use udp::{DnsResponse, ResolveWithBuffer};

fn main() -> Result<(), Box<dyn Error>> {
    let address = std::env::args().nth(2);
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_size, source)) => {
                let query = DnsResponse::from_bytes((&buf, 0))?.1;

                if let Some(addr) = address.as_ref() {
                    let _resolver_socket =
                        UdpSocket::bind(addr).expect("Failed to bind resolver address");
                    let mut _buf = [0; 512];
                    // let mut answers = Vec::new();

                    for question in query.questions {
                        let _single_query = DnsResponse {
                            header: query.header.clone(),
                            questions: vec![question],
                            answers: vec![],
                        };
                    }
                } else {
                    let response = query.resolve(&buf)?.to_expected();

                    let response = response.to_bytes()?;
                    udp_socket.send_to(&response, source)?;
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
    Ok(())
}
