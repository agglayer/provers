use aggkit_prover::version;
use anyhow::Context as _;
use clap::Parser as _;
use sp1_sdk::HashableKey as _;
use sp1_zkvm::lib::utils::words_to_bytes_le;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = aggkit_prover::cli::Cli::parse();

    match cli.cmd {
        aggkit_prover::cli::Commands::Run { config_path } => {
            aggkit_prover::runtime(config_path, &version())?
        }
        aggkit_prover::cli::Commands::Config => {
            println!(
                "{}",
                toml::to_string_pretty(&aggkit_prover_config::ProverConfig::default())
                    .context("Failed to serialize ProverConfig to TOML")?
            );
        }
        aggkit_prover::cli::Commands::ValidateConfig { config_path } => {
            match aggkit_prover_config::ProverConfig::try_load(config_path.as_path()) {
                Ok(config) => {
                    println!(
                        "{}",
                        toml::to_string_pretty(&config)
                            .context("Failed to serialize ProverConfig to TOML")?
                    );
                }
                Err(error) => eprintln!("{}", error),
            }
        }
        aggkit_prover::cli::Commands::Vkey => {
            let vkey =
                prover_executor::Executor::get_vkey(aggchain_proof_service::AGGCHAIN_PROOF_ELF);

            println!(
                "aggchain_proof_vkey: 0x{}",
                hex::encode(words_to_bytes_le(&vkey.hash_u32()))
            );
        }
    }

    Ok(())
}
