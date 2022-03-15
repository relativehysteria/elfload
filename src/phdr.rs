//! The ELF program header
use std::{
    io::{BufReader, Read},
    fs::File,
};
use crate::{
    err::Error,
    consume,
};

/// The ELF program header
#[derive(Debug)]
pub struct ProgramHeader {
    pub r#type: u32,
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
        let r#type   = consume!(reader, u32)?;
        let flags    = consume!(reader, u32)?;
        let offset   = consume!(reader, usize)?;
        let vaddr    = consume!(reader, usize)?;
        let paddr    = consume!(reader, usize)?;
        let filesz   = consume!(reader, usize)?;
        let memsz    = consume!(reader, usize)?;
        let align    = consume!(reader, usize)?;

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
