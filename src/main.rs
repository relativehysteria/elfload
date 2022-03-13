use elfload::{
    parse::*,
    util::*,
};

extern "C" {
    fn mmap(addr: *mut u8, length: usize, prot: i32,
            flags: i32, fd: i32, offset: i32) -> *mut u8;
    fn mprotect(addr: *const u8, len: usize, prot: u32) -> i32;
}

// TODO:
//     * Unsafe validation
//     * Better way to jump to the entry point
fn main() {
    // Pointers to loaded segments in memory
    let mut loaded = Vec::new();

    // Entry point
    let mut entry: *const u8 = std::ptr::null();

    {
        // Parse all the segments
        let phdrs = parse_elf("samples/hi_there").unwrap();

        // Only get the loadable ones
        let phdrs = phdrs.into_iter()
            .filter(|hdr| hdr.r#type == 1)
            .collect::<Vec<ProgramHeader>>();

        // Load the segments to memory
        for phdr in phdrs.iter() {
            // Get the page size
            let page_size = (get_page_size() - 1) as u64;

            // Prepare the protections for the current allocation of the data:
            // PROT_READ | PROT_WRITE.
            // We are going to change this to the flags defined by the phdr
            // later on when we have the region loaded to memory.
            let prot = 1 | 2;

            // Prepare the mmap flags: MAP_PRIVATE | MAP_ANONYMOUS
            let mmap_flags = 2 | 32;

            // Make sure the buffer is allocated in a page of its own.
            let capacity = ((phdr.memsz & (!page_size)) + page_size) as usize;

            // Map the buffer into memory
            let vaddr = phdr.vaddr as *mut u8;
            let ptr = unsafe { mmap(vaddr, capacity, prot, mmap_flags, -1, 0) };

            // Copy the data into it
            unsafe {
                let dst = std::slice::from_raw_parts_mut(ptr, phdr.data.len());
                dst.copy_from_slice(&phdr.data[..]);
            }

            // Check if we have the entry point
            if phdr.flags & 1 == 1 {
                entry = ptr;
            }

            // Change the protection of the mapping to the one specified
            // by the phdr.
            unsafe { mprotect(ptr, capacity, switch_rx(phdr.flags)); }

            loaded.push(ptr);
        }
    }

    // Jump to the entry point
    unsafe {
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        let entry: fn() = std::mem::transmute(entry);
        entry();
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
        std::arch::asm!("nop");
    }
}
