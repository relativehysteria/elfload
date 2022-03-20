//! The ELF program header
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    fs::File,
};
use num_enum::TryFromPrimitive;
use crate::{
    Error,
    consume,
};


/// Contents of a section
#[derive(Debug)]
pub enum SectionContents {
    Dynamic(Vec<DynamicEntry>),
}

impl SectionContents {
    /// Parse data as a `SectionContents::Dynamic`
    pub fn parse_dynamic(data: &[u8]) -> Result<Self, Error> {
        // Each entry should be 16 bytes in size
        let entry_size = 16usize;

        // Check that the data is of valid size
        if data.len() % entry_size != 0 {
            return Err(Error::InvalidDataSize(data.len()));
        }

        // Prepare a vector for dynamic entries
        let mut entries = Vec::with_capacity(data.len() / entry_size);

        // Traverse each entry and push it into the `entries` vec
        for entry in data.chunks(entry_size) {
            let val = usize::from_le_bytes(entry[8..].try_into().unwrap());
            let tag = usize::from_le_bytes(entry[..8].try_into().unwrap());
            let tag = DynamicTag::try_from(tag)
                .map_err(|e| Error::InvalidDynamicTag(e.number))?;
            entries.push(DynamicEntry { tag, val });
        }

        Ok(Self::Dynamic(entries))
    }
}

/// Entry in a dynamic section
#[derive(Debug)]
pub struct DynamicEntry {
    pub tag: DynamicTag,
    pub val: usize,
}

/// Tags that can show up in `DynamicEntry`
#[derive(Debug, TryFromPrimitive, PartialEq, Eq)]
#[repr(usize)]
pub enum DynamicTag {
    Null        = 0x0,
    Needed      = 0x1,
    PltRelSz    = 0x2,
    PltGot      = 0x3,
    Hash        = 0x4,
    StrTab      = 0x5,
    SymTab      = 0x6,
    Rela        = 0x7,
    RelaSz      = 0x8,
    RelaEnt     = 0x9,
    StrSz       = 0xA,
    SymEnt      = 0xB,
    Init        = 0xC,
    Fini        = 0xD,
    SoName      = 0xE,
    RPath       = 0xF,
    Symbolic    = 0x10,
    Rel         = 0x11,
    RelSz       = 0x12,
    RelEnt      = 0x13,
    PltRel      = 0x14,
    Debug       = 0x15,
    TextRel     = 0x16,
    JmpRel      = 0x17,
    BindNow     = 0x18,
    InitArray   = 0x19,
    FiniArray   = 0x1A,
    InitArraySz = 0x1B,
    FiniArraySz = 0x1C,
    Flags       = 0x1E,
    LoOs        = 0x6000_0000,
    HiOs        = 0x6FFF_FFFF,
    LoProc      = 0x7000_0000,
    HiProc      = 0x7FFF_FFFF,
    GnuHash     = 0x6FFF_FEF5,
    Flags1      = 0x6FFF_FFFB,
    RelaCount   = 0x6FFF_FFF9,
}

/// Different types of defined segments
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
    pub r#type:   SegmentType,
    pub flags:    u32,
    pub offset:   usize,
    pub vaddr:    usize,
    pub paddr:    usize,
    pub filesz:   usize,
    pub memsz:    usize,
    pub align:    usize,
    pub contents: Option<SectionContents>
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

        // Load the contents if we want to
        let contents = match r#type {
            SegmentType::Dynamic => {
                // Prepare a buffer for the contents
                let mut contents = vec![0u8; filesz];

                // Save the current position.
                let pos = reader.stream_position().map_err(Error::Seek)?;

                // Seek to the offset.
                reader.seek(SeekFrom::Start(offset as u64))
                    .map_err(Error::Seek)?;

                // Red to the buffer.
                reader.read_exact(&mut contents).map_err(Error::Read)?;

                // Seek back
                reader.seek(SeekFrom::Start(pos)).map_err(Error::Seek)?;

                // Attempt to parse the contents
                Some(SectionContents::parse_dynamic(&mut contents)?)
            },
            _ => None,
        };

        Ok(Self {
            r#type,
            flags,
            offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            align,
            contents,
        })
    }
}
