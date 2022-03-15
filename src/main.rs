use elfload::{
    ProgramHeader,
    err::Error,
    parse::*,
    util::*,
    constants::*,
    memory::*,
};

const BASE: usize = 0x400_000;

fn main() {
    // Make sure that we are running as a 64-bit binary
    if !(cfg!(target_pointer_width = "64")) {
        return;
    }

    // Pointers to loaded segments in memory
    let mut loaded = Vec::new();

    // Entry point. `valid_entry` is changed later on when we validate that the
    // ep actually points to an executable section
    let entry: *const u8;
    let mut valid_entry = false;

    {
        // Parse all the segments
        let (ep, phdrs) = parse_elf("samples/hi_there_pie").unwrap();
        entry = (ep + BASE) as *const u8;

        // Only get the loadable ones
        let phdrs = phdrs.into_iter()
            .filter(|hdr| hdr.r#type == PT_LOAD)
            .filter(|hdr| hdr.memsz != 0)
            .collect::<Vec<ProgramHeader>>();

        // Load the segments to memory
        for phdr in phdrs.iter() {
            // Map the buffer into memory
            let vaddr   = page_align(phdr.vaddr);
            let padding = phdr.vaddr - vaddr;
            let len     = padding + phdr.memsz;
            let vaddr   = (vaddr + BASE) as *mut u8;
            let ptr = memmap(vaddr as *const u8, len).unwrap();

            // Copy the data into it
            unsafe {
                let dst = std::slice::from_raw_parts_mut(vaddr.add(padding),
                                                         phdr.memsz);
                dst.copy_from_slice(&phdr.data[..]);
            }

            // If this section is executable, check if the ep points to it.
            // If the ep doesn't point to any executable section, it is invalid.
            let flags = switch_rx(phdr.flags);
            if flags & PROT_EXEC == PROT_EXEC {
                if !valid_entry {
                    let entry = entry as usize;
                    let ptr   = ptr as usize;
                    valid_entry = entry >= ptr && entry < (ptr + len);
                }
            }

            // Change the protection of the mapping to the one specified
            // by the phdr.
            memprotect(ptr, len, flags).unwrap();

            loaded.push(ptr);
        }
    }

    // Jump to the entry point
    if valid_entry {
        unsafe {
            let entry: fn() = std::mem::transmute(entry);
            entry();
        }
    } else {
        Err(Error::InvalidEntry).unwrap()
    }
}
