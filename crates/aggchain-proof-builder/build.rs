pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let elf_path = agglayer_elf_build::build_program("crates/aggchain-proof-program")?;
    eprintln!("ELF_PATH={elf_path}");
    Ok(())
}
