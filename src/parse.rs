use std::{
    path::Path,
    io::{BufReader, Read},
    fs::File,
    mem::size_of,
};
use crate::err::Error;

/// Read bytes from a reader
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

/// Parse an ELF from disk
pub fn parse_elf(path: impl AsRef<Path>) -> Result<(), Error> {
    // Open the file
    let mut reader =
        BufReader::new(File::open(path).map_err(|e| Error::Open(e))?);

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
    let _     = consume!(reader, 17)?;
    let entry = consume!(reader, u64)?;

    // Get the program header table offset
    let phoff = consume!(reader, u64)?;

    // Skip straight to the number of program headers
    let _     = consume!(reader, 16)?;
    let phcnt = consume!(reader, u16)?;

    println!("Entry point:                 0x{:x?}", entry);
    println!("Program header table offset: 0x{:x?}", phoff);
    println!("Number of program headers:   {:?}",    phcnt);

    Ok(())
}
