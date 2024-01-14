use deku::{DekuContainerWrite, DekuEnumExt, DekuError, DekuRead, DekuUpdate, DekuWrite};
use derivative::Derivative;

#[derive(Clone, PartialEq, DekuRead, DekuWrite, Derivative)]
#[derivative(Default(new = "true"))]
#[derivative(Debug)]
pub struct Header {
    #[deku(bytes = "2", endian = "big")]
    #[derivative(Default(value = "1234"))]
    id: u16,

    pub qr_indicator: QRIndicator,

    #[deku(bits = "4")]
    #[derivative(Default(value = "0"))]
    op_code: u8,

    #[deku(bits = "1")]
    #[derivative(Default(value = "false"))]
    authoritative_answer: bool,

    #[deku(bits = "1")]
    #[derivative(Default(value = "false"))]
    /// always 0 for udp
    truncation: bool,

    #[deku(bits = "1")]
    #[derivative(Default(value = "false"))]
    /// always 0 for udp
    recursion_available: bool,

    #[deku(bits = "3")]
    #[derivative(Default(value = "0"))]
    reserved: u8,

    #[deku(bits = "4", pad_bits_after = "1")]
    #[derivative(Default(value = "0"))]
    response_code: u8,

    #[deku(bytes = "2", endian = "big")]
    #[derivative(Default(value = "0"))]
    pub question_count: u16,

    #[deku(bytes = "2", endian = "big")]
    #[derivative(Default(value = "0"))]
    answer_record_count: u16,

    #[deku(bytes = "2", endian = "big")]
    #[derivative(Default(value = "0"))]
    authority_record_count: u16,

    #[deku(bytes = "2", endian = "big")]
    #[derivative(Default(value = "0"))]
    additional_record_count: u16,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
#[derive(Derivative)]
#[derivative(Default)]
pub enum QRIndicator {
    #[deku(id = "0x00")]
    Query,

    #[deku(id = "0x01")]
    #[derivative(Default)]
    Response,
}

#[cfg(test)]
mod test {
    use deku::{
        bitvec::{bits, Lsb0},
        DekuContainerRead,
    };
    use std::error::Error;

    use super::*;

    #[test]
    fn default_header() -> Result<(), Box<dyn Error>> {
        let header = Header::default();
        let header_bits = header.to_bits()?;
        let header_bytes = header.to_bytes()?;

        assert_eq!(header_bytes.len(), 12);
        assert_eq!(header_bytes, [4, 210, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            header_bits,
            bits![
                0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        );

        let header_from_bytes = Header::from_bytes((header_bytes.as_ref(), 0))?.1;
        assert_eq!(header, header_from_bytes);

        let mut header_from = Header::try_from(header_bytes.as_ref())?;
        assert_eq!(header, header_from);

        header_from.question_count = 1;
        dbg!(header_from.to_bytes()?);
        Ok(())
    }
}
