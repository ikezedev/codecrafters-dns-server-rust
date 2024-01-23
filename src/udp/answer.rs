use deku::{DekuContainerWrite, DekuEnumExt, DekuError, DekuRead, DekuUpdate, DekuWrite};
use derivative::Derivative;

use super::name::Name;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite, Derivative)]
#[derivative(Default)]
pub struct Answer {
    pub name: Name,
    r#type: AnswerType,
    class: AnswerClass,

    #[deku(bytes = 4, endian = "big")]
    #[derivative(Default(value = "60"))]
    ttl: u32,

    #[deku(update = "self.data.len()", endian = "big")]
    #[derivative(Default(value = "4"))]
    length: u16,

    #[deku(count = "length")]
    #[derivative(Default(value = "vec![8_u8, 8, 8, 8]"))]
    data: Vec<u8>,
}

impl Answer {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "big")]
#[derive(Derivative)]
#[derivative(Default)]
pub enum AnswerClass {
    #[deku(id = "1")]
    #[derivative(Default)]
    IN = 1,

    #[deku(id = "2")]
    CS,

    #[deku(id = "3")]
    CH,

    #[deku(id = "4")]
    HS,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "big")]
#[derive(Derivative)]
#[derivative(Default)]
pub enum AnswerType {
    #[deku(id = "1")]
    #[derivative(Default)]
    A = 1,

    #[deku(id = "2")]
    NS,

    #[deku(id = "3")]
    MD,

    #[deku(id = "4")]
    MF,

    #[deku(id = "5")]
    CNAME,

    #[deku(id = "6")]
    SOA,

    #[deku(id = "7")]
    MB,

    #[deku(id = "8")]
    MG,

    #[deku(id = "9")]
    MR,

    #[deku(id = "10")]
    NULL,

    #[deku(id = "11")]
    WKS,

    #[deku(id = "12")]
    PTR,

    #[deku(id = "13")]
    HINFO,

    #[deku(id = "14")]
    MINFO,

    #[deku(id = "15")]
    MX,

    #[deku(id = "16")]
    TXT,
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use super::*;

    #[test]
    fn default_test() -> Result<(), Box<dyn Error>> {
        let ans_default = Answer::default();
        let ans_bytes = ans_default.to_bytes()?;

        let ans = Answer::try_from(ans_bytes.as_ref())?;

        assert_eq!(ans, ans_default);
        Ok(())
    }
}
