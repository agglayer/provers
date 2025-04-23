use aggchain_proof_service::AGGCHAIN_VKEY_SELECTOR;
use aggkit_prover::version;
use anyhow::Context as _;
use clap::Parser as _;
use prover_config::{CpuProverConfig, ProverType};
use sp1_sdk::HashableKey as _;

fn main() -> anyhow::Result<()> {
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
            let executor = prover_executor::Executor::new(
                &ProverType::CpuProver(CpuProverConfig::default()),
                &None,
                aggchain_proof_service::AGGCHAIN_PROOF_ELF,
            );
            let vkey = executor.get_vkey();
            let vkey_hex = hex::encode(vkey.hash_bytes());
            println!("0x{}", vkey_hex.trim_start_matches("0x"));
        }

        aggkit_prover::cli::Commands::VkeySelector => {
            let vkey_selector_hex =
                hex::encode(AGGCHAIN_VKEY_SELECTOR.to_be_bytes());
            println!("0x{}", vkey_selector_hex.trim_start_matches("0x"));
        }
    }

    Ok(())
}
