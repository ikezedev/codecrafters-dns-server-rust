mod answer;
mod header;
mod name;
mod question;

use crate::traits::{ReadUntill, WriteAll};
use deku::{DekuContainerWrite, DekuError, DekuRead, DekuUpdate, DekuWrite};

use self::answer::Answer;
pub use self::header::{Header, QRIndicator};
pub use self::question::Question;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite, Default)]
pub struct Dns {
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

impl ResolveWithBuffer for Dns {
    fn resolve(self, buf: &[u8]) -> Result<Dns, DekuError> {
        let questions = self.questions.resolve(buf)?;

        Ok(Dns { questions, ..self })
    }
}

pub trait ResolveWithBuffer {
    fn resolve(self, buf: &[u8]) -> Result<Self, DekuError>
    where
        Self: Sized;
}

impl<T> ResolveWithBuffer for Vec<T>
where
    T: ResolveWithBuffer,
{
    fn resolve(self, buf: &[u8]) -> Result<Vec<T>, DekuError> {
        self.into_iter()
            .map(|item| item.resolve(buf))
            .collect::<Result<Vec<_>, _>>()
    }
}

impl Dns {
    pub fn to_expected(mut self) -> Self {
        self.header.response_code = if self.header.op_code == 0 { 0 } else { 4 };
        self.header.qr_indicator = QRIndicator::Response;
        self.header.question_count = self.questions.len() as u16;
        self.header.answer_record_count = self.answers.len() as u16;

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

        let dns = Dns {
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
        let dns_query = Dns::from_bytes((&input, 0))?;

        assert_eq!(
            dns_bytes,
            [header_bytes, question_bytes, answer_bytes].concat()
        );

        let resolved = dns_query.1.resolve(&[])?;

        assert_eq!(
            dns,
            Dns {
                answers: vec![Answer::default()],
                ..resolved
            }
        );

        Ok(())
    }
}
