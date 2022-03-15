//! Logic to parse a file and its program headers
use std::{
    io::{BufReader, Read, SeekFrom, Seek},
    fs::File,
};
use crate::{
    err::Error,
    ProgramHeader,
};

/// Read bytes from a reader
#[macro_export]
macro_rules! consume {
    // Read a single u8
    ($reader:expr) => {{
        let mut bytes = [0u8; 1];
        $reader.read_exact(&mut bytes).map(|_| {
            bytes[0]
        }).map_err(|e| Error::Read(e))
    }};

    // Read a single `$type`
    ($reader:expr, $type:ty) => {{
        use std::mem::size_of;
        let mut bytes = [0u8; size_of::<$type>()];
        $reader.read_exact(&mut bytes).map(|_| {
            <$type>::from_le_bytes(bytes)
        }).map_err(|e| Error::Read(e))
    }};

    // Read `$size` amount of bytes
    ($reader:expr, $size:expr) => {{
        let mut bytes = [0u8; $size];
        $reader.read_exact(&mut bytes).map(|_| {
            bytes
        }).map_err(|e| Error::Read(e))
    }};
}

/// Parse an ELF from disk.
///
/// Returns (entry_point, Vec<ProgramHeader>)
pub fn parse_elf(reader: &mut BufReader<File>)
    -> Result<(usize, Vec<ProgramHeader>), Error> {
    // Verify the ELF magic
    if &consume!(reader, 4)? != b"\x7FELF" {
        return Err(Error::InvalidMagic);
    }

    // Verify the bitness (64b is expected)
    if consume!(reader)? != 2 {
        return Err(Error::InvalidBits);
    }

    // Verify the endianness (little endian is expected)
    if consume!(reader)? != 1 {
        return Err(Error::InvalidEndian);
    }

    // Verify the version
    if consume!(reader)? != 1 {
        return Err(Error::InvalidVersion);
    }

    // Skip straight to the entry point
    let _____ = consume!(reader, 17)?;
    let entry = consume!(reader, usize)?;

    // Get the program header table offset
    let phoff = consume!(reader, usize)?;

    // Skip straight to the number of program headers
    let _____ = consume!(reader, 16)?;
    let phcnt = consume!(reader, u16)?;

    // Seek to the program headers
    reader.seek(SeekFrom::Start(phoff as u64)).map_err(Error::SeekPhdr)?;

    // Parse the headers
    let mut phdrs = Vec::new();
    for _ in 0..phcnt {
        phdrs.push(ProgramHeader::parse(reader)?);
    }

    Ok((entry, phdrs))
}
