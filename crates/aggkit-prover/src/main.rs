use aggchain_proof_service::AGGCHAIN_VKEY_SELECTOR;
use aggkit_prover::version;
use clap::Parser as _;
use eyre::Context as _;
use sp1_sdk::HashableKey as _;

fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    let cli = aggkit_prover::cli::Cli::parse();

    match cli.cmd {
        aggkit_prover::cli::Commands::Run { config_path } => {
            aggkit_prover::runtime(config_path, &version())?
        }
        aggkit_prover::cli::Commands::Config => {
            let config = toml::to_string_pretty(&aggkit_prover_config::ProverConfig::default())
                .context("Failed to serialize ProverConfig to TOML")?;

            println!("{config}");
        }
        aggkit_prover::cli::Commands::ValidateConfig { config_path } => {
            match aggkit_prover_config::ProverConfig::try_load(config_path.as_path()) {
                Ok(config) => {
                    let config = toml::to_string_pretty(&config)
                        .context("Failed to serialize ProverConfig to TOML")?;

                    println!("{config}");
                }
                Err(error) => eprintln!("{error}"),
            }
        }
        aggkit_prover::cli::Commands::Vkey => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(async move {
                    let vkey = prover_executor::Executor::compute_program_vkey(
                        aggchain_proof_service::AGGCHAIN_PROOF_ELF,
                    )
                    .await?;

                    let vkey_hex = hex::encode(vkey.hash_bytes());
                    println!("0x{vkey_hex}");
                    Ok::<(), eyre::Report>(())
                })?;
        }

        aggkit_prover::cli::Commands::VkeySelector => {
            let vkey_selector_hex = hex::encode(AGGCHAIN_VKEY_SELECTOR.to_be_bytes());
            println!("0x{vkey_selector_hex}");
        }

        aggkit_prover::cli::Commands::OpSuccinctVkey { elf_dir } => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(async move {
                    // The CLI is short-lived, so leaking the ELF bytes to satisfy the
                    // `'static` bound of `compute_program_vkey` is acceptable.
                    let read_elf = |name: &str| -> eyre::Result<&'static [u8]> {
                        let path = elf_dir.join(name);
                        let bytes = std::fs::read(&path)
                            .with_context(|| format!("Reading ELF {}", path.display()))?;
                        let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                        Ok(leaked)
                    };

                    let aggregation_vkey = prover_executor::Executor::compute_program_vkey(
                        read_elf("aggregation-elf")?,
                    )
                    .await?;
                    let range_vkey = prover_executor::Executor::compute_program_vkey(read_elf(
                        "range-elf-embedded",
                    )?)
                    .await?;

                    let aggregation_vkey_bytes =
                        aggkit_prover_types::vkey::encode_verifying_key(&aggregation_vkey)?;

                    println!("[aggchain-proof-service.op-succinct]");
                    println!(
                        "aggregation-vkey = \"0x{}\"",
                        hex::encode(&aggregation_vkey_bytes)
                    );
                    println!(
                        "range-vkey-commitment = \"0x{}\"",
                        hex::encode(range_vkey.hash_bytes())
                    );
                    println!(
                        "# aggregation on-chain vkey hash (bytes32): 0x{}",
                        hex::encode(aggregation_vkey.bytes32_raw())
                    );
                    Ok::<(), eyre::Report>(())
                })?;
        }
    }

    Ok(())
}
