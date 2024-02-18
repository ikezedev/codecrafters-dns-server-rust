mod answer;
mod header;
mod name;
mod question;

use crate::traits::{ReadUntill, WriteAll};
use deku::{DekuContainerWrite, DekuError, DekuRead, DekuUpdate, DekuWrite};

use self::answer::Answer;
pub use self::header::{Header, QRIndicator};
pub use self::question::Question;
use self::question::QuestionQuery;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite, Default)]
pub struct DnsResponse {
    pub header: Header,
    #[deku(
        writer = "Question::write_all(deku::output, &self.questions)",
        reader = "Question::read_count(deku::rest, header.question_count)"
    )]
    pub questions: Vec<Question>,
    #[deku(
        writer = "Answer::write_all(deku::output, &self.answers)",
        reader = "Answer::read_count(deku::rest, header.answer_record_count)"
    )]
    pub answers: Vec<Answer>,
}

#[derive(Debug, Clone, PartialEq, DekuRead, Default)]
pub struct DnsQuery {
    pub header: Header,
    #[deku(reader = "QuestionQuery::read_until_null(deku::rest)")]
    pub questions: Vec<QuestionQuery>,
}

impl ResolveWithBuffer<DnsResponse> for DnsResponse {
    fn resolve(self, buf: &[u8]) -> Result<DnsResponse, DekuError> {
        let questions = self.questions.resolve(buf)?;
        let answers: Vec<_> = questions
            .iter()
            .map(|q| Answer::new(q.domain_name.clone()))
            .collect();

        let mut header = self.header;
        header.question_count = questions.len() as u16;
        header.answer_record_count = answers.len() as u16;

        Ok(DnsResponse {
            header,
            questions,
            answers,
        })
    }
}

pub trait ResolveWithBuffer<T> {
    fn resolve(self, buf: &[u8]) -> Result<T, DekuError>;
}

impl<T, U> ResolveWithBuffer<Vec<U>> for Vec<T>
where
    T: ResolveWithBuffer<U>,
{
    fn resolve(self, buf: &[u8]) -> Result<Vec<U>, DekuError> {
        self.into_iter()
            .map(|item| item.resolve(buf))
            .collect::<Result<Vec<_>, _>>()
    }
}

impl DnsResponse {
    pub fn to_expected(mut self) -> Self {
        self.header.response_code = if self.header.op_code == 0 { 0 } else { 4 };
        self.header.qr_indicator = QRIndicator::Response;

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use deku::{DekuContainerRead, DekuContainerWrite};
    use pretty_assertions::assert_eq;
    use std::error::Error;

    #[test]
    fn test_dns_bytes() -> Result<(), Box<dyn Error>> {
        let mut header = Header::new();
        let question = Question::new("google.com", 1);
        let answer = Answer::default();
        header.answer_record_count = 1;
        header.question_count = 1;

        let header_bytes = header.to_bytes()?;
        let question_bytes = question.to_bytes()?;
        let answer_bytes = answer.to_bytes()?;

        let dns = DnsResponse {
            header,
            questions: vec![question],
            answers: vec![answer],
        };
        let dns_bytes = dns.to_bytes()?;
        let input = [
            header_bytes.clone(),
            question_bytes.clone(),
            answer_bytes.clone(),
        ]
        .concat();
        let dns_query = DnsResponse::from_bytes((&input, 0))?;

        assert_eq!(
            dns_bytes,
            [header_bytes, question_bytes, answer_bytes].concat()
        );

        let resolved = dns_query.1.resolve(&[])?;

        assert_eq!(
            dns,
            DnsResponse {
                answers: vec![Answer::default()],
                ..resolved
            }
        );

        Ok(())
    }
}
