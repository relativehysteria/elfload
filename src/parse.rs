//! Logic to parse a file and its program headers

use std::{
    path::Path,
    io::{BufReader, Read, SeekFrom, Seek},
    fs::File,
    mem::size_of,
};
use crate::{
    err::Error,
    util::*,
};

extern "C" {
    fn mmap(addr: *mut u8, length: usize, prot: i32,
            flags: i32, fd: i32, offset: i32) -> *mut u8;
    fn mprotect(addr: *const u8, len: usize, prot: u32) -> i32;
}

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

/// Parse an ELF from disk.
///
/// Returns (entry_point, Vec<ProgramHeader>)
pub fn parse_elf(path: impl AsRef<Path>) -> Result<Vec<ProgramHeader>, Error> {
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
    let _      = consume!(reader, 17)?;
    let _entry = consume!(reader, u64)?;

    // Get the program header table offset
    let phoff = consume!(reader, u64)?;

    // Skip straight to the number of program headers
    let _     = consume!(reader, 16)?;
    let phcnt = consume!(reader, u16)?;

    // Seek to the program headers
    reader.seek(SeekFrom::Start(phoff)).map_err(Error::SeekPhdr)?;

    // Parse the headers
    let mut phdrs = Vec::new();
    for _ in 0..phcnt {
        phdrs.push(ProgramHeader::parse(&mut reader)?);
    }

    Ok(phdrs)
}


/// The ELF program header
#[derive(Debug)]
pub struct ProgramHeader {
    pub r#type: u32,
    pub flags:  u32,
    pub offset: u64,
    pub vaddr:  u64,
    pub paddr:  u64,
    pub filesz: u64,
    pub memsz:  u64,
    pub align:  u64,

    /// Data assigned to this program header.
    /// From `offset` to `filesz`
    pub data: Vec<u8>,
}

// TODO: A lot of unsafe code here, so be sure to validate it
impl ProgramHeader {
    /// Parse a header from the `reader`.
    ///
    /// This function expects that the `reader` is already positioned
    /// at the beginning of the header.
    pub fn parse(reader: &mut BufReader<File>) -> Result<Self, Error> {
        // Parse the header
        let r#type   = consume!(reader, u32)?;
        let flags    = consume!(reader, u32)?;
        let offset   = consume!(reader, u64)?;
        let vaddr    = consume!(reader, u64)?;
        let paddr    = consume!(reader, u64)?;
        let filesz   = consume!(reader, u64)?;
        let memsz    = consume!(reader, u64)?;
        let align    = consume!(reader, u64)?;

        // Get the page size
        let page_size = (get_page_size() - 1) as u64;

        // Prepare the protections for the current allocation of the data:
        // PROT_READ | PROT_WRITE.
        // We are going to change this to the flags defined by the phdr later on
        // when we have the region loaded to memory.
        let prot = 1 | 2;

        // Prepare the mmap flags: MAP_PRIVATE | MAP_ANONYMOUS
        let mmap_flags = 2 | 32;

        // Prepare the data buffer. Make sure that it is allocated in a page
        // of its own.
        let capacity = ((memsz & (!page_size)) + page_size) as usize;

        // Create the data Vec
        let mut data = unsafe {
            let ptr = mmap(vaddr as *mut u8, capacity, prot, mmap_flags, -1, 0);
            Vec::from_raw_parts(ptr, 0, capacity)
        };

        if filesz > 0 {
            // Save the current stream position
            let pos = reader.stream_position().map_err(Error::SeekData)?;

            // Resize the vector so that we can read exactly `filesz`
            data.resize(filesz as usize, 0u8);

            // Seek to the header's data section
            reader.seek(SeekFrom::Start(offset)).map_err(Error::SeekData)?;
            reader.read_exact(&mut data).map_err(|e| Error::Read(e))?;

            // Seek back to the end of the header
            reader.seek(SeekFrom::Start(pos)).map_err(Error::SeekData)?;
        }

        // Resize the buffer from `filesz` to `memsz`
        data.resize(memsz as usize, 0u8);

        // Change the protection of the mapping to the one specified
        // by the phdr. Everything is aligned, so we don't need to check it.
        unsafe { mprotect(data.as_ptr(), data.len(), switch_rx(flags)); }

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
