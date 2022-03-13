extern "C" {
    fn sysconf(name: u32) -> u32;
}

/// Returns the current's system page size
pub fn get_page_size() -> u64 {
    (unsafe { sysconf(30) }) as u64
}

/// Align an address to the upper page boundary
pub fn page_align(addr: u64) -> u64 {
    addr & !(get_page_size() as u64 - 1)
}

/// This functions switches flags from `RWX` to `XWR` and vice-versa.
///
/// The flags in the program header are as follows: `RWX`, whereas the flags
/// expected by mprotect are `XWR`.
pub fn switch_rx(flags: u32) -> u32 {
    let r = 1 << 2;
    let r = ((flags & r == r) as u32) << 0;

    let w = 1 << 1;
    let w = ((flags & w == w) as u32) << 1;

    let x = 1 << 0;
    let x = ((flags & x == x) as u32) << 2;

    x^w^r
}
