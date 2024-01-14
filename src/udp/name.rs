use std::io::Read;

use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct Name {
    #[deku(
        reader = "Name::read(deku::rest)",
        writer = "Name::write(deku::output, &self.string)"
    )]
    string: String,
}

impl Name {
    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, String), DekuError> {
        let mut acc: Vec<String> = Vec::new();

        let mut next = rest;
        loop {
            let (rest, len) = u8::read(next, ())?;
            if len == 0 {
                return Ok((rest, acc.join(".")));
            }

            let mut buf = String::with_capacity(len.into());
            let mut handle = rest.take(len.into());
            handle
                .read_to_string(&mut buf)
                .map_err(|err| DekuError::Parse(err.to_string()))?;

            next = handle.into_inner();

            acc.push(buf);
        }
    }

    fn write(output: &mut BitVec<u8, Msb0>, string: &str) -> Result<(), DekuError> {
        for label in string.split(".") {
            (label.len() as u8).write(output, ())?;
            label.as_bytes().write(output, ())?;
        }
        0_u8.write(output, ())
    }

    pub fn new(name: &str) -> Self {
        Self {
            string: name.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    #[test]
    fn write() -> Result<(), Box<dyn Error>> {
        let name = Name::new("google.com");
        let name_bytes = name.to_bytes()?;
        assert_eq!(
            name_bytes,
            &[0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 00,]
        );
        Ok(())
    }

    #[test]
    fn read() -> Result<(), Box<dyn Error>> {
        let bytes = &[
            0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 00,
        ];
        let name = Name::from_bytes((bytes, 0))?.1;
        assert_eq!(name, Name::new("google.com"));

        let name = Name::try_from(bytes.as_ref())?;
        assert_eq!(name, Name::new("google.com"));
        Ok(())
    }
}
