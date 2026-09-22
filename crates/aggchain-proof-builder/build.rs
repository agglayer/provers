use agglayer_elf_build::ProgramBuilder;

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // `ProgramBuilder` always emits `AGGLAYER_ELF_PATH`, so the noop program is
    // built first and re-exported under its own variable; the real program is
    // built last so `AGGLAYER_ELF_PATH` keeps pointing at it.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let noop_elf_path = ProgramBuilder::new("crates/aggchain-proof-noop-program")?
        .cached_elf_path(format!(
            "{manifest_dir}/elf/noop/riscv64im-succinct-zkvm-elf"
        ))
        .run()?;
    println!("cargo::rustc-env=AGGLAYER_NOOP_ELF_PATH={noop_elf_path}");
    eprintln!("NOOP_ELF_PATH={noop_elf_path}");

    let elf_path = agglayer_elf_build::build_program("crates/aggchain-proof-program")?;
    eprintln!("ELF_PATH={elf_path}");
    Ok(())
}
