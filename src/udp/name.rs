use std::io::Read;

use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

use super::ResolveWithBuffer;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct Name {
    #[deku(
        reader = "Name::read(deku::rest)",
        writer = "Name::write(deku::output, &self.string)"
    )]
    string: String,
}

#[derive(Debug, Clone, PartialEq, DekuRead)]
pub struct NameRead {
    #[deku(reader = "NameRead::read(deku::rest)")]
    data: NameEntry,
}

impl NameRead {
    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, NameEntry), DekuError> {
        let mut labels = Vec::<Label>::new();
        let mut data = rest;

        loop {
            let (rest, len) = u8::read(data, ())?;
            if len == 0 {
                return Ok((rest, NameEntry::Labels(labels)));
            }

            match len >> 6 {
                0b11 => {
                    let (rest, len) = u8::read(rest, ())?;
                    let pointer = Pointer(len);
                    if labels.is_empty() {
                        return Ok((rest, NameEntry::Pointer(pointer)));
                    } else {
                        return Ok((
                            rest,
                            NameEntry::Combined {
                                start: labels,
                                end: pointer,
                            },
                        ));
                    }
                }
                0b00 => {
                    let mut buf = String::with_capacity(len.into());
                    let mut handle = rest.take(len.into());
                    handle
                        .read_to_string(&mut buf)
                        .map_err(|err| DekuError::Parse(err.to_string()))?;

                    data = handle.into_inner();

                    labels.push(Label(buf));
                }
                _ => unreachable!("labels must start with 0b11 or 0b00"),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label(String);

#[derive(Debug, Clone, PartialEq)]
pub struct Pointer(u8);

impl ResolveWithBuffer<Name> for Pointer {
    fn resolve(self, buf: &[u8]) -> Result<Name, DekuError> {
        Ok(Name::from_bytes((buf, (self.0 * 8).into()))?.1)
    }
}

impl ResolveWithBuffer<Name> for NameRead {
    fn resolve(self, buf: &[u8]) -> Result<Name, DekuError> {
        match self.data {
            NameEntry::Pointer(p) => p.resolve(buf),
            NameEntry::Labels(labels) => Ok(Name::new(
                &labels
                    .iter()
                    .map(|l| l.0.to_string())
                    .collect::<Vec<_>>()
                    .join("."),
            )),
            NameEntry::Combined { start, end } => {
                let labels = start
                    .iter()
                    .map(|l| l.0.to_string())
                    .collect::<Vec<_>>()
                    .join(".");

                let pointer_label = end.resolve(buf)?;
                let resolved = format!("{labels}.{}", pointer_label.string);
                Ok(Name::new(&resolved))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NameEntry {
    Pointer(Pointer),
    Labels(Vec<Label>),
    Combined { start: Vec<Label>, end: Pointer },
}

impl Default for Name {
    fn default() -> Self {
        Self::new("codecrafters.io")
    }
}

impl From<Name> for NameRead {
    fn from(value: Name) -> Self {
        let labels: Vec<_> = value
            .string
            .split(".")
            .map(|s| Label(s.to_string()))
            .collect();
        Self {
            data: NameEntry::Labels(labels),
        }
    }
}

impl Default for NameRead {
    fn default() -> Self {
        Name::new("codecrafters.io").into()
    }
}

impl Name {
    fn read<'a, 'b>(
        rest: &'b BitSlice<u8, Msb0>,
    ) -> Result<(&'b BitSlice<u8, Msb0>, String), DekuError> {
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

        let name3 = NameRead::try_from(bytes.as_ref())?;
        dbg!(name3);
        Ok(())
    }
}
