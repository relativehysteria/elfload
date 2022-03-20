use std::{
    io::{Read, SeekFrom, Seek},
};
use elfload::{
    Error,
    phdr::*,
    parse::*,
    util::*,
    constants::*,
    memory::*,
};

const BASE: usize = 0x400_000;
const NAME: &str  = "samples/hi_there_pie";

fn main() {
    // Make sure that we are running as a 64-bit binary
    if !(cfg!(target_pointer_width = "64")) {
        return;
    }

    // Pointers to loaded segments in memory
    let mut loaded = Vec::new();

    // This is changed later on when we validate that the
    // ep actually points to an executable section
    let mut valid_entry = false;

    let entry = {
        // Parse the file
        let mut elf = ELF::parse(NAME).unwrap();

        // Add the `BASE` to the entry point
        elf.entry += BASE;

        // Get the file's loadable segments
        let loadable = elf.phdrs.iter()
            .inspect(|hdr| println!("{hdr:x?}"))
            .filter(|hdr| matches!(hdr.r#type, SegmentType::Load))
            .filter(|hdr| hdr.memsz != 0)
            .collect::<Vec<&ProgramHeader>>();

        // Load the segments to memory
        for phdr in loadable.iter() {
            // Map the buffer into memory
            let vaddr   = page_align(phdr.vaddr);
            let padding = phdr.vaddr - vaddr;
            let len     = padding + phdr.memsz;
            let vaddr   = (vaddr + BASE) as *mut u8;
            let ptr = memmap(vaddr as *const u8, len).unwrap();

            // If there is any data in the file, copy it into the buffer
            if phdr.filesz > 0 {
                // Seek to the data section
                elf.reader.seek(SeekFrom::Start(phdr.offset as u64))
                    .map_err(Error::Seek)
                    .unwrap();

                // Copy the data into the buffer
                unsafe {
                    let mut dst = std::slice::from_raw_parts_mut(
                        vaddr.add(padding),
                        phdr.memsz
                    );
                    elf.reader.read_exact(&mut dst)
                        .map_err(|e| Error::Read(e))
                        .unwrap();
                }
            }

            // If this section is executable, check if the ep points to it.
            // If the ep doesn't point to any executable section, it is invalid.
            let flags = switch_rx(phdr.flags);
            if flags & PROT_EXEC == PROT_EXEC {
                if !valid_entry {
                    let ptr = ptr as usize;
                    valid_entry = elf.entry >= ptr && elf.entry < (ptr + len);
                }
            }

            // Change the protection of the mapping to the one specified
            // by the phdr.
            memprotect(ptr, len, flags).unwrap();

            loaded.push(ptr);
        }

        elf.entry as *const u8
    };

    // Jump to the entry point
    if valid_entry {
        unsafe {
            let entry: fn() = std::mem::transmute(entry);
            entry();
        }
    } else {
        Err(Error::InvalidEntry(entry)).unwrap()
    }
}
