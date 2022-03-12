use elfload::{
    parse::*,
    err::Error,
};

extern "C" {
    fn mprotect(addr: *const u8, len: usize, prot: u32) -> i32;
    fn sysconf(name: u32) -> u32;
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

            // Set the permissions
            mprotect(ptr, seg.data.len(), seg.flags);
        }

        println!("Before jump!");
        let entry: fn() = std::mem::transmute(exec.data.as_ptr());
        entry();
        println!("After jump!");
    }
}
