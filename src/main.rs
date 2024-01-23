mod udp;

use std::{error::Error, net::UdpSocket};

use deku::{DekuContainerRead, DekuContainerWrite};
use udp::{DnsQuery, ResolveWithBuffer};

fn main() -> Result<(), Box<dyn Error>> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_size, source)) => {
                let query = DnsQuery::from_bytes((&buf, 0))?.1;
                let response = query.resolve(&buf)?;
                let response = response.to_expected();

                let response = response.to_bytes()?;
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
