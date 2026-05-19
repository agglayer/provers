use std::{env, fs, path::Path};

use cargo_metadata::{camino::Utf8Path, MetadataCommand};
use eyre::{eyre, Context, OptionExt};
use sp1_build::BuildArgs;

const BUILD_ENV_VAR: &str = "AGGLAYER_ELF_BUILD";
const CACHED_ELF_PATH: &str = "elf/riscv64im-succinct-zkvm-elf";
const PROGRAM_DIR: &str = "crates/aggchain-proof-program";

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    println!("cargo::rerun-if-env-changed={BUILD_ENV_VAR}");

    let elf_path = match build_mode()? {
        BuildMode::Cached => cached_elf_path()?,
        BuildMode::Build(extra_args) => build_program(extra_args)?,
        BuildMode::Update(extra_args) => {
            let elf_path = build_program(extra_args)?;
            copy_elf(&elf_path, &cached_elf_path()?).context("Copying zkvm ELF to cache")?;
            elf_path
        }
    };

    println!("cargo::rerun-if-changed={elf_path}");
    println!("cargo::rustc-env=AGGLAYER_ELF_PATH={elf_path}");
    eprintln!("ELF_PATH={elf_path}");

    Ok(())
}

enum BuildMode {
    Cached,
    Build(Vec<String>),
    Update(Vec<String>),
}

fn build_mode() -> eyre::Result<BuildMode> {
    let value = match env::var(BUILD_ENV_VAR) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(BuildMode::Cached),
        Err(error) => eyre::bail!("Malformed mode from {BUILD_ENV_VAR}: {error}"),
    };

    let mut parts = value.split_whitespace();
    let mode = parts.next().unwrap_or("cached");
    let extra_args = parts.map(str::to_owned).collect();

    match mode {
        "" | "cached" => Ok(BuildMode::Cached),
        "build" => Ok(BuildMode::Build(extra_args)),
        "refresh" | "update" => Ok(BuildMode::Update(extra_args)),
        mode => eyre::bail!("Unrecognized mode {mode:?}"),
    }
}

fn cached_elf_path() -> eyre::Result<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").context("Cannot obtain manifest dir")?;
    Ok(Utf8Path::new(&manifest_dir)
        .join(CACHED_ELF_PATH)
        .to_string())
}

fn build_program(extra_args: Vec<String>) -> eyre::Result<String> {
    let host_metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Host workspace metadata extraction failed")?;
    let program_dir = host_metadata.workspace_root.join(PROGRAM_DIR);

    let program_metadata = MetadataCommand::new()
        .no_deps()
        .current_dir(&program_dir)
        .exec()
        .context("Program workspace metadata extraction failed")?;

    let mut build_args = BuildArgs {
        docker: true,
        workspace_directory: Some(host_metadata.workspace_root.to_string()),
        ..Default::default()
    };
    let extra_args = std::iter::once("cargo-prove-build".to_owned()).chain(extra_args);
    clap::Parser::update_from(&mut build_args, extra_args);

    let elf_path = built_elf_path(&program_metadata, &build_args)?;
    sp1_build::build_program_with_args(program_dir.as_str(), build_args);

    Ok(elf_path)
}

fn built_elf_path(
    program_metadata: &cargo_metadata::Metadata,
    build_args: &BuildArgs,
) -> eyre::Result<String> {
    let mut paths = sp1_build::generate_elf_paths(program_metadata, Some(build_args))
        .map_err(|error| eyre!(error))
        .context("Failed to extract zkvm ELF paths")?
        .into_iter();

    let (_package, path) = paths.next().ok_or_eyre("No zkvm ELF paths")?;
    eyre::ensure!(paths.next().is_none(), "Too many zkvm ELF paths");

    Ok(path.to_string())
}

fn copy_elf(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> eyre::Result<()> {
    if let Some(cached_elf_dir) = destination.as_ref().parent() {
        fs::create_dir_all(cached_elf_dir).context("Failed to create directory for zkvm ELF")?;
    }

    let source_file_name = source
        .as_ref()
        .file_name()
        .ok_or_eyre("Built zkvm ELF path has no file name")?;
    let temporary_path = Path::new(&env::var("OUT_DIR").context("Getting build OUT_DIR")?)
        .join(source_file_name)
        .with_extension("temporary");
    fs::copy(source, temporary_path.as_path()).context("Failed to copy zkvm ELF")?;
    fs::rename(temporary_path, destination).context("Failed to move zkvm ELF")?;

    Ok(())
}
