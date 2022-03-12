//! Errors returned by this crate

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

    /// An error occurred while seeking for the program headers
    SeekPhdr(std::io::Error),

    /// An error occurred while seeking for some data
    SeekData(std::io::Error),

    /// Not a single executable section was found
    NoExec,
}
