use agglayer_elf_build::ProgramBuilder;

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // `ProgramBuilder` always emits `AGGLAYER_ELF_PATH`, so the mock program is
    // built first and re-exported under its own variable; the real program is
    // built last so `AGGLAYER_ELF_PATH` keeps pointing at it.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let mock_elf_path = ProgramBuilder::new("crates/aggchain-proof-mock-program")?
        .cached_elf_path(format!(
            "{manifest_dir}/elf/mock/riscv64im-succinct-zkvm-elf"
        ))
        .run()?;
    println!("cargo::rustc-env=AGGLAYER_MOCK_ELF_PATH={mock_elf_path}");
    eprintln!("MOCK_ELF_PATH={mock_elf_path}");

    let elf_path = agglayer_elf_build::build_program("crates/aggchain-proof-program")?;
    eprintln!("ELF_PATH={elf_path}");
    Ok(())
}
