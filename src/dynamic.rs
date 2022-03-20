//! Dynamic segment structs and funcs
use num_enum::TryFromPrimitive;
use crate::Error;

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

/// Struct holding data used to handle relative relocations etc as defined by
/// `Elf_Rela`
///
/// `r_info` is split into `r#type` and `sym`
#[derive(Debug)]
pub struct Rela {
    pub offset: usize,
    pub r#type: u32,
    pub symbol: u32,
    pub addend: usize,
}

impl Rela {
    /// Parse data as `Rela`
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        // Check if we have a valid data size
        if data.len() != 24 {
            return Err(Error::InvalidDataSize(data.len()));
        }

        // Parse the data
        let offset = usize::from_le_bytes(data[..8].try_into().unwrap());
        let r#type = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let symbol = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let addend = usize::from_le_bytes(data[16..].try_into().unwrap());

        Ok(Self {
            offset,
            r#type,
            symbol,
            addend,
        })
    }
}
