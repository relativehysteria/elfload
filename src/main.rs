use elfload::{
    parse::*,
    util::*,
    constants::*,
};

extern "C" {
    fn mmap(addr: *mut u8, length: usize, prot: u32,
            flags: u32, fd: u32, offset: u32) -> *mut u8;
    fn mprotect(addr: *const u8, len: usize, prot: u32) -> i32;
}

const BASE: usize = 0x400_000;

// TODO:
//     * Unsafe validation
//     * Better way to jump to the entry point
fn main() {
    // Make sure that we are running as a 64-bit binary
    if !(cfg!(target_pointer_width = "64")) {
        return;
    }

    // Pointers to loaded segments in memory
    let mut loaded = Vec::new();

    // Entry point
    let mut entry: *const u8 = std::ptr::null();

    {
        // Parse all the segments
        let phdrs = parse_elf("samples/hi_there_pie").unwrap();

        // Only get the loadable ones
        let phdrs = phdrs.into_iter()
            .filter(|hdr| hdr.r#type == PT_LOAD)
            .filter(|hdr| hdr.memsz != 0)
            .collect::<Vec<ProgramHeader>>();

        // Load the segments to memory
        for phdr in phdrs.iter() {
            // Prepare the protections for the current allocation of the data:
            // PROT_READ | PROT_WRITE.
            let prot = PROT_READ | PROT_WRITE;

            // Prepare the mmap flags: MAP_PRIVATE | MAP_ANONYMOUS
            let mmap_flags = MAP_PRIVATE | MAP_ANONYMOUS;

            // Map the buffer into memory
            let vaddr   = page_align(phdr.vaddr);
            let padding = phdr.vaddr - vaddr;
            let len     = padding + phdr.memsz;
            let vaddr   = (vaddr + BASE) as *mut u8;
            let ptr = unsafe { mmap(vaddr, len, prot, mmap_flags, !0, 0) };

            // Copy the data into it
            unsafe {
                let dst = std::slice::from_raw_parts_mut(vaddr.add(padding),
                                                         phdr.data.len());
                dst.copy_from_slice(&phdr.data[..]);
            }

            // Check if we have the entry point
            let flags = switch_rx(phdr.flags);
            if flags & PROT_EXEC == PROT_EXEC {
                entry = ptr;
            }

            // Change the protection of the mapping to the one specified
            // by the phdr.
            unsafe { mprotect(ptr, phdr.memsz, flags); }

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
