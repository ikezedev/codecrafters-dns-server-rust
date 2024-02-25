use deku::{DekuContainerWrite, DekuRead, DekuUpdate, DekuWrite};
use derivative::Derivative;

use super::{name::Name, ResolveWithBuffer};

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite, Derivative)]
#[derivative(Default)]
pub struct Question {
    pub domain_name: Name,

    #[deku(bytes = "2", endian = "big")]
    #[derivative(Default(value = "1"))]
    q_type: u16,

    #[deku(bytes = "2", endian = "big")]
    #[derivative(Default(value = "1"))]
    class: u16,
}

impl Question {
    pub fn new(name: &str, q_type: u16) -> Self {
        Self {
            domain_name: Name::new(name),
            q_type,
            class: 1,
        }
    }
}

impl ResolveWithBuffer for Question {
    fn resolve(self, buf: &[u8]) -> Result<Question, deku::DekuError> {
        Ok(Question {
            domain_name: self.domain_name.resolve(buf)?,
            q_type: self.q_type,
            class: self.class,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use deku::{DekuContainerRead, DekuContainerWrite};
    use std::error::Error;

    #[test]
    fn test_question_bytes() -> Result<(), Box<dyn Error>> {
        let question = Question::new("codecrafters.io", 1);
        let question_bytes = question.to_bytes()?;

        let (_, question_from_bytes) = Question::from_bytes((question_bytes.as_ref(), 0))?;
        assert_eq!(question, question_from_bytes);

        let question_try_from = Question::try_from(question_bytes.as_ref())?;
        assert_eq!(question, question_try_from);

        Ok(())
    }
}
