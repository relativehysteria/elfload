/// Errors returned by this crate
#[derive(Debug)]
pub enum Error {
    /// Failed to read a field from the input
    Read(std::io::Error),

    /// Failed to open the file
    Open(std::io::Error),

    /// Failed to parse ELF magic
    InvalidMagic,

    /// We are not parsing a 64bit binary
    InvalidBits,

    /// We are not parsing a little endian binary
    InvalidEndian,

    /// Invalid ELF version
    InvalidVersion,

    /// Invalid segment type in a program header
    InvalidSegmentType(u32),

    /// Invalid dynamic tag was found
    InvalidDynamicTag(usize),

    /// An error has occurred while seeking somewhere
    Seek(std::io::Error),

    /// Not a single executable section was found
    NoExec,

    /// A generic error has occurred while executing `mmap`
    Mmap(*const u8, usize),

    /// A generic error has occurred while executing `mprotect`
    Mprotect(*const u8, usize, u32),

    /// The specified entry point is invalid
    InvalidEntry(*const u8),

    /// A generic error thrown when the data isn't of expected size
    InvalidDataSize(usize),
}
