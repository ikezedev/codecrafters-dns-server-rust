use deku::{
    bitvec::{BitSlice, Msb0},
    prelude::*,
};

use crate::traits::DekuRes;

use super::{ReadUntill, ResolveWithBuffer, WriteAll};

impl ResolveWithBuffer for Name {
    fn resolve(self, buf: &[u8]) -> Result<Name, DekuError> {
        if let Some(NameKind::Pointer(start)) = self.entries.last() {
            let resolved_pointer = Name::from_bytes((buf, (start * 8).into()))?.1;
            if self.entries.len() == 1 {
                Ok(resolved_pointer)
            } else {
                let len = self.entries.len() - 1;
                Ok(Name {
                    entries: self
                        .entries
                        .into_iter()
                        .take(len)
                        .chain(resolved_pointer.entries.into_iter())
                        .collect(),
                })
            }
        } else {
            Ok(self)
        }
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "2")]
pub enum NameKind {
    #[deku(id = "0b00")]
    Label {
        #[deku(bits = "6")]
        count: u8,
        #[deku(count = "count")]
        data: Vec<u8>,
    },

    #[deku(id = "0b11")]
    Pointer(#[deku(bits = "14", endian = "big")] u16),
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct Name {
    #[deku(
        reader = "NameKind::read_until_null_or_pointer(deku::rest)",
        writer = "NameKind::write_all_with_null(deku::output, &self.entries)"
    )]
    entries: Vec<NameKind>,
}

impl Name {
    pub fn new(name: &str) -> Self {
        Self {
            entries: name
                .split(".")
                .map(|label| NameKind::Label {
                    count: label.len() as u8,
                    data: label.into(),
                })
                .collect(),
        }
    }
}

impl NameKind {
    fn read_until_null_or_pointer(rest: &BitSlice<u8, Msb0>) -> DekuRes<Self> {
        Self::read_internal(
            rest,
            |res| {
                res.last()
                    .map(|last| match last {
                        NameKind::Label { .. } => false,
                        NameKind::Pointer(_) => true,
                    })
                    .unwrap_or_default()
            },
            |zero| zero == 0,
        )
    }
}

impl Default for Name {
    fn default() -> Self {
        Self::new("codecrafters.io")
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
        let (_, name) = Name::from_bytes((bytes, 0))?;
        assert_eq!(name, Name::new("google.com"));

        let name = Name::try_from(bytes.as_ref())?;
        assert_eq!(name, Name::new("google.com"));
        Ok(())
    }

    #[test]
    fn pointer() -> Result<(), Box<dyn Error>> {
        let bytes = &[
            1, 70, 3, 73, 83, 73, 4, 65, 82, 80, 65, 0, 3, 70, 79, 79, 192, 0, 192, 6,
        ];
        let (input, first) = Name::from_bytes((bytes, 0))?;
        assert_eq!(first, Name::new("F.ISI.ARPA"));

        let (input, second) = Name::from_bytes(input)?;
        let second = second.resolve(bytes.as_ref())?;
        assert_eq!(second, Name::new("FOO.F.ISI.ARPA"));

        let (_, third) = Name::from_bytes(input)?;
        let third = third.resolve(bytes.as_ref())?;
        assert_eq!(third, Name::new("ARPA"));

        Ok(())
    }
}
