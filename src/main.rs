use elfload::{
    parse::*,
    err::Error,
};

extern "C" {
    fn mprotect(addr: *const u8, len: usize, prot: u32) -> i32;
    fn sysconf(name: u32) -> u32;
}

/// This functions switches flags from `RWX` to `XWR` and vice-versa.
///
/// The flags in the program header are as follows: `RWX`, whereas the flags
/// expected by mprotect are `XWR`.
fn switch_rx(flags: u32) -> u32 {
    let r = 1 << 2;
    let r = ((flags & r == r) as u32) << 0;

    let w = 1 << 1;
    let w = ((flags & w == w) as u32) << 1;

    let x = 1 << 0;
    let x = ((flags & x == x) as u32) << 2;

    x^w^r
}

fn main() {
    // Parse all the segments
    let phdrs = parse_elf("test_file").unwrap();

    // Only get the loadable ones
    let phdrs = phdrs.into_iter()
        .filter(|hdr| hdr.r#type == 1)
        .collect::<Vec<ProgramHeader>>();

    // Find the code segment
    let exec = phdrs.iter()
        .find(|hdr| hdr.flags & 1 == 1)
        .ok_or(Error::NoExec)
        .expect("No code segment found..");

    unsafe {
        // Get the page size
        let page_size = (sysconf(30) - 1) as u64;

        for seg in phdrs.iter() {
            // Page align the pointer to the data
            let ptr = seg.data.as_ptr() as u64;
            let ptr = ptr & (!page_size);
            let ptr = ptr as *const u8;

            // Switch from the phdr flag format to the one expected by mprotect
            let flags = switch_rx(seg.flags);

            // Set the permissions
            mprotect(ptr, seg.data.len(), flags);
        }

        println!("Before jump!");
        let entry: fn() = std::mem::transmute(exec.data.as_ptr());
        entry();
        println!("After jump!");
    }
}
