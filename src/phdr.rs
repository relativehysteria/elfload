//! The ELF program header
use std::{
    io::{BufReader, Read, SeekFrom, Seek},
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

    /// Data assigned to this program header.
    /// From `offset` to `filesz`
    pub data: Vec<u8>,
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
        let mut data = Vec::new();

        if filesz > 0 {
            // Save the current stream position
            let pos = reader.stream_position().map_err(Error::SeekData)?;

            // Resize the vector so that we can read exactly `filesz`
            data.resize(filesz, 0u8);

            // Seek to the header's data section
            reader.seek(SeekFrom::Start(offset as u64))
                .map_err(Error::SeekData)?;
            reader.read_exact(&mut data).map_err(|e| Error::Read(e))?;

            // Seek back to the end of the header
            reader.seek(SeekFrom::Start(pos)).map_err(Error::SeekData)?;
        }

        // Resize the buffer from `filesz` to `memsz`
        data.resize(memsz, 0u8);

        Ok(Self {
            r#type,
            flags,
            offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            align,
            data
        })
    }
}
