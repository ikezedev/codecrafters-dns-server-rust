mod answer;
mod header;
mod name;
mod question;

use deku::{DekuContainerWrite, DekuRead, DekuUpdate, DekuWrite};

use self::answer::Answer;
pub use self::header::{Header, QRIndicator};
pub use self::question::Question;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite, Default)]
pub struct DnsResponse {
    pub header: Header,
    pub question: Question,
    pub answer: Answer,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite, Default)]
pub struct DnsQuery {
    pub header: Header,
    pub question: Question,
}

impl From<DnsQuery> for DnsResponse {
    fn from(value: DnsQuery) -> Self {
        let mut header = Header::default();

        header.id = value.header.id;
        header.op_code = value.header.op_code;
        header.recursion_desired = value.header.recursion_desired;
        header.response_code = if value.header.op_code == 0 { 0 } else { 4 };

        header.question_count = 1;
        header.qr_indicator = QRIndicator::Response;
        header.answer_record_count = 1;

        let mut question = Question::default();
        question.domain_name = value.question.domain_name.clone();

        let mut answer = Answer::default();
        answer.name = value.question.domain_name;

        Self {
            answer,
            header,
            question,
        }
    }
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

        let dns = DnsResponse {
            header,
            question,
            answer,
        };
        let dns_bytes = dns.to_bytes()?;
        let dns_from = DnsResponse::try_from(dns_bytes.as_ref())?;

        assert_eq!(dns, dns_from);

        assert_eq!(
            dns_bytes,
            [header_bytes, question_bytes, answer_bytes].concat()
        );
        Ok(())
    }
}
