use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

use crate::version;

/// Aggkit prover command line interface.
#[derive(Parser)]
#[command(version = version())]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run {
        /// The path to the configuration file.
        #[arg(long, short, value_hint = ValueHint::FilePath, default_value = "aggkit-prover.toml", env = "CONFIG_PATH")]
        config_path: PathBuf,
    },

    Config,

    ValidateConfig {
        /// The path to the aggkit-prover configuration file.
        #[arg(value_hint = ValueHint::FilePath)]
        config_path: PathBuf,
    },

    /// Proof verification key.
    Vkey,

    /// Proof verification key selector.
    VkeySelector,
}
