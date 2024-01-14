mod header;
mod name;
mod question;

use deku::{DekuContainerWrite, DekuRead, DekuUpdate, DekuWrite};

pub use self::header::{Header, QRIndicator};
pub use self::question::Question;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct Dns {
    pub header: Header,
    pub question: Question,
}

#[cfg(test)]
mod test {
    use super::*;
    use deku::DekuContainerWrite;
    use std::error::Error;

    #[test]
    fn test_dns_bytes() -> Result<(), Box<dyn Error>> {
        let header = Header::new();
        let question = Question::new("google.com", 1);
        let header_bytes = header.to_bytes()?;
        let question_bytes = question.to_bytes()?;

        let dns = Dns { header, question };
        let dns_bytes = dns.to_bytes()?;
        let dns_from = Dns::try_from(dns_bytes.as_ref())?;

        assert_eq!(dns, dns_from);

        assert_eq!(dns_bytes, [header_bytes, question_bytes].concat());
        Ok(())
    }
}
