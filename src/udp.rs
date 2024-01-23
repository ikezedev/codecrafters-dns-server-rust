mod answer;
mod header;
mod name;
mod question;

use core::fmt;

use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::{DekuContainerRead, DekuContainerWrite, DekuError, DekuRead, DekuUpdate, DekuWrite};

use self::answer::Answer;
pub use self::header::{Header, QRIndicator};
pub use self::question::Question;
use self::question::QuestionQuery;

#[derive(Debug, Clone, PartialEq, DekuWrite, Default)]
pub struct DnsResponse {
    pub header: Header,
    #[deku(writer = "Question::write_all(deku::output, &self.questions)")]
    pub questions: Vec<Question>,
    #[deku(writer = "Answer::write_all(deku::output, &self.answers)")]
    pub answers: Vec<Answer>,
}

#[derive(Debug, Clone, PartialEq, DekuRead, Default)]
pub struct DnsQuery {
    pub header: Header,
    #[deku(reader = "QuestionQuery::read_until_null(deku::rest)")]
    pub questions: Vec<QuestionQuery>,
}

impl ResolveWithBuffer<DnsResponse> for DnsQuery {
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

pub trait ReadUntill<'a>: DekuRead<'a> + DekuContainerRead<'a> {
    fn read_till_end(
        rest: &'a BitSlice<u8, Msb0>,
    ) -> Result<(&BitSlice<u8, Msb0>, Vec<Self>), DekuError>
    where
        Self: Sized + fmt::Debug,
    {
        let mut res = Vec::<Self>::new();
        let mut next = rest;

        loop {
            if next.is_empty() {
                return Ok((next, res));
            }
            let (input, val) = Self::read(next, ())?;
            res.push(val);
            next = input;
        }
    }

    fn read_until_null(
        rest: &'a BitSlice<u8, Msb0>,
    ) -> Result<(&'a BitSlice<u8, Msb0>, Vec<Self>), DekuError>
    where
        Self: Sized + fmt::Debug,
    {
        let mut res = Vec::<Self>::new();
        let mut next = rest;

        loop {
            if next.is_empty() || u8::read(next, ())?.1 == 0 {
                return Ok((next, res));
            }
            let (input, val) = Self::read(next, ())?;
            res.push(val);
            next = input;
        }
    }
}

pub trait WriteAll: DekuWrite + DekuContainerWrite {
    fn write_all(output: &mut BitVec<u8, Msb0>, entries: &[Self]) -> Result<(), DekuError>
    where
        Self: Sized,
    {
        for entry in entries {
            entry.to_bytes()?.write(output, ())?;
        }
        Ok(())
    }
}

impl<'a, T> ReadUntill<'a> for T where T: DekuRead<'a> + DekuContainerRead<'a> {}
impl<T> WriteAll for T where T: DekuWrite + DekuContainerWrite {}

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
    use deku::DekuContainerWrite;
    use pretty_assertions::assert_eq;
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
            questions: vec![question],
            answers: vec![answer],
        };
        let dns_bytes = dns.to_bytes()?;

        let dns_query = DnsQuery::try_from(
            [header_bytes.clone(), question_bytes.clone()]
                .concat()
                .as_ref(),
        )?;
        let resolved = dns_query.resolve(&[])?;
        assert_eq!(
            dns,
            DnsResponse {
                answers: vec![Answer::default()],
                ..resolved
            }
        );

        assert_eq!(
            dns_bytes,
            [header_bytes, question_bytes, answer_bytes].concat()
        );
        Ok(())
    }
}
