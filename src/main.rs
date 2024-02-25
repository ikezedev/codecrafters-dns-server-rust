mod traits;
mod udp;

use std::{error::Error, net::UdpSocket};

use deku::{DekuContainerRead, DekuContainerWrite};
use udp::{Dns, ResolveWithBuffer};

fn main() -> Result<(), Box<dyn Error>> {
    let address = std::env::args().nth(2);
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        if let Ok((_size, source)) = udp_socket.recv_from(&mut buf) {
            let query = Dns::from_bytes((&buf, 0))?.1;

            let addr = address.as_ref().unwrap();
            let mut answers = Vec::new();

            for question in &query.questions {
                let mut header = query.header.clone();
                header.question_count = 1;

                let question = question.clone().resolve(&buf)?;

                let single_query = Dns {
                    header,
                    questions: vec![question],
                    answers: vec![],
                }
                .to_bytes()?;

                udp_socket.send_to(&single_query, addr)?;

                let mut inner_buf = [0; 512];

                if let Ok(_) = udp_socket.recv_from(&mut inner_buf) {
                    let query1 = Dns::from_bytes((&inner_buf, 0))?.1;
                    answers.extend(query1.answers);
                }
            }
            let response = Dns { answers, ..query }.resolve(&buf)?.to_expected();
            let response = response.to_bytes()?;
            udp_socket.send_to(&response, source)?;
        }
    }
}
