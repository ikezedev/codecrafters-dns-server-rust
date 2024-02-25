use std::fmt;

use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    DekuContainerRead, DekuContainerWrite, DekuError, DekuRead, DekuWrite,
};

pub type DekuRes<'a, T> = Result<(&'a BitSlice<u8, Msb0>, Vec<T>), DekuError>;

pub trait ReadUntill<'a>: DekuRead<'a> + DekuContainerRead<'a> {
    fn read_until_null(rest: &'a BitSlice<u8, Msb0>) -> DekuRes<Self>
    where
        Self: Sized + fmt::Debug,
    {
        Self::read_internal(rest, |_| false, |zero| zero == 0)
    }

    fn read_count(rest: &'a BitSlice<u8, Msb0>, len: impl Into<usize>) -> DekuRes<'a, Self>
    where
        Self: Sized + fmt::Debug,
    {
        let len = len.into();
        Self::read_internal(rest, |res| res.len() == len, |_| false)
    }

    fn read_internal(
        rest: &'a BitSlice<u8, Msb0>,
        f: impl Fn(&[Self]) -> bool,
        f2: impl Fn(u8) -> bool,
    ) -> DekuRes<Self>
    where
        Self: Sized + fmt::Debug,
    {
        let mut res = Vec::<Self>::new();
        let mut next = rest;

        loop {
            if next.is_empty() || f(&res) {
                return Ok((next, res));
            }
            if next.is_empty() {
                return Ok((next, res));
            }

            let (end_next, zero) = u8::read(next, ())?;
            if f2(zero) {
                return Ok((end_next, res));
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

    fn write_all_with_null(output: &mut BitVec<u8, Msb0>, entries: &[Self]) -> Result<(), DekuError>
    where
        Self: Sized,
    {
        Self::write_all(output, entries)?;
        if !entries.is_empty() {
            0_u8.write(output, ())?;
        }
        Ok(())
    }
}

impl<'a, T> ReadUntill<'a> for T where T: DekuRead<'a> + DekuContainerRead<'a> {}
impl<T> WriteAll for T where T: DekuWrite + DekuContainerWrite {}
