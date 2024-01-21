mod answer;
mod header;
mod name;
mod question;

use deku::{DekuContainerWrite, DekuRead, DekuUpdate, DekuWrite};

use self::answer::Answer;
pub use self::header::{Header, QRIndicator};
pub use self::question::Question;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite, Default)]
pub struct Dns {
    pub header: Header,
    pub question: Question,
    pub answer: Answer,
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
        let answer = Answer::default();

        let header_bytes = header.to_bytes()?;
        let question_bytes = question.to_bytes()?;
        let answer_bytes = answer.to_bytes()?;

        let dns = Dns {
            header,
            question,
            answer,
        };
        let dns_bytes = dns.to_bytes()?;
        let dns_from = Dns::try_from(dns_bytes.as_ref())?;

        assert_eq!(dns, dns_from);

        assert_eq!(
            dns_bytes,
            [header_bytes, question_bytes, answer_bytes].concat()
        );
        Ok(())
    }
}
