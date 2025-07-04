pub fn main() -> agglayer_elf_build::Result<()> {
    agglayer_elf_build::build_program("crates/aggchain-proof-program").map(|elf_path| {
        eprintln!("ELF_PATH={elf_path}");
    })
}
