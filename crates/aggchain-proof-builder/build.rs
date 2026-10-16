use agglayer_elf_build::ProgramBuilder;

const SP1_GUEST_DOCKER_TAG: &str =
    "v6.2.2@sha256:af8c2f46bb5e417c1c2fb939cd4397c75aa4f2439a9fd7c1f9ce2e0a693f6f6b";

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // `ProgramBuilder` always emits `AGGLAYER_ELF_PATH`, so the noop program is
    // built first and re-exported under its own variable; the real program is
    // built last so `AGGLAYER_ELF_PATH` keeps pointing at it.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let noop_elf_path = ProgramBuilder::new("crates/aggchain-proof-noop-program")?
        .docker_tag(SP1_GUEST_DOCKER_TAG)
        .cached_elf_path(format!(
            "{manifest_dir}/elf/noop/riscv64im-succinct-zkvm-elf"
        ))
        .run()?;
    println!("cargo::rustc-env=AGGLAYER_NOOP_ELF_PATH={noop_elf_path}");
    eprintln!("NOOP_ELF_PATH={noop_elf_path}");

    let elf_path = ProgramBuilder::new("crates/aggchain-proof-program")?
        .docker_tag(SP1_GUEST_DOCKER_TAG)
        .run()?;
    eprintln!("ELF_PATH={elf_path}");
    Ok(())
}
