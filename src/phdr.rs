//! The ELF program header
use std::{
    io::{BufReader, Read},
    fs::File,
};
use num_enum::TryFromPrimitive;
use crate::{
    err::Error,
    consume,
};

/// Segment types
#[derive(Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum SegmentType {
    Null        = 0x0,
    Load        = 0x1,
    Dynamic     = 0x2,
    Interp      = 0x3,
    Note        = 0x4,
    Shlib       = 0x5,
    PhdrTable   = 0x6,
    Loos        = 0x6000_0000,
    Hios        = 0x6FFF_FFFF,
    LoProc      = 0x7000_0000,
    HiProc      = 0x7FFF_FFFF,
    GnuEhFrame  = 0x6474_E550,
    GnuStack    = 0x6474_E551,
    GnuRelRo    = 0x6474_E552,
    GnuProperty = 0x6474_E553,
}

/// The ELF program header
#[derive(Debug)]
pub struct ProgramHeader {
    pub r#type: SegmentType,
    pub flags:  u32,
    pub offset: usize,
    pub vaddr:  usize,
    pub paddr:  usize,
    pub filesz: usize,
    pub memsz:  usize,
    pub align:  usize,
}

impl ProgramHeader {
    /// Parse a header from the `reader`.
    ///
    /// This function expects that the `reader` is already positioned
    /// at the beginning of the header.
    pub fn parse(reader: &mut BufReader<File>) -> Result<Self, Error> {
        // Parse the header
        let r#type = consume!(reader, u32)?;
        let flags  = consume!(reader, u32)?;
        let offset = consume!(reader, usize)?;
        let vaddr  = consume!(reader, usize)?;
        let paddr  = consume!(reader, usize)?;
        let filesz = consume!(reader, usize)?;
        let memsz  = consume!(reader, usize)?;
        let align  = consume!(reader, usize)?;

        // Convert the bytes into a type
        let r#type = SegmentType::try_from(r#type)
            .map_err(|e| Error::InvalidSegmentType(e.number))?;

        Ok(Self {
            r#type,
            flags,
            offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            align,
        })
    }
}
