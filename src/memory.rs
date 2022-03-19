use crate::{
    err::Error,
    constants::*,
};

extern "C" {
    fn mmap(addr: *const u8, length: usize, prot: u32,
            flags: u32, fd: u32, offset: u32) -> *const u8;
    fn mprotect(addr: *const u8, len: usize, prot: u32) -> i32;
}

/// Creates a `PROT_READ | PROT_WRITE`, `MAP_PRIVATE | MAP_ANONYMOUS` mapping in
/// memory. If you want to change the protections, use `memprotect`
pub fn memmap(addr: *const u8, length: usize) -> Result<*const u8, Error> {
    let prot  = PROT_READ   | PROT_WRITE;
    let flags = MAP_PRIVATE | MAP_ANONYMOUS;

    let ptr = unsafe { mmap(addr, length, prot, flags, !0, 0) };

    if ptr != addr {
        Err(Error::Mmap(addr, length))
    } else {
        Ok(ptr)
    }

}

/// Changes the memory protections of an `addr`
pub fn memprotect(addr: *const u8, length: usize, prot: u32) -> Result<(), Error> {
    match unsafe { mprotect(addr, length, prot) } {
        -1 => Err(Error::Mprotect(addr, length, prot)),
        __ => Ok(()),
    }
}
