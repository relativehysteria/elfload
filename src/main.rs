use elfload::{
    parse::*,
    err::Error,
};

// TODO:
//     * Unsafe validation
//     * Better way to jump to the entry point
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

    // Jump to the entry point
    unsafe {
        let entry: fn() = std::mem::transmute(exec.data.as_ptr());
        entry();
    }
}
