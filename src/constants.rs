pub const PT_LOAD: u32 = 1;

pub const PROT_READ: u32  = 1 << 0;
pub const PROT_WRITE: u32 = 1 << 1;
pub const PROT_EXEC: u32  = 1 << 2;

pub const MAP_PRIVATE: u32   = 1 << 1;
pub const MAP_ANONYMOUS: u32 = 1 << 5;
