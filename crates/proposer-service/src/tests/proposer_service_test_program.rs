use std::{str::FromStr, sync::Arc};

use alloy_primitives::B256;
use clap::Parser;
use proposer_client::{config::ProposerClientConfig, FepProposerRequest, GrpcUri};
use proposer_service::{config::ProposerServiceConfig, ProposerService};
use prover_alloy::L1RpcEndpoint;
use prover_logger::log::Log;
use tower::{Service, ServiceExt};
use tracing::info;
use url::Url;

/// Proposer service test program, retrieving the proof from the prover service.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Start block (last proven block before the requested proof)
    #[arg(short, long)]
    last_proven_block: u64,

    /// Requested end block
    #[arg(short, long)]
    requested_end_block: u64,

    /// L1 block hash
    #[arg(short = 'H', long)]
    l1_block_hash: String,

    /// JSON-RPC endpoint of the l1 node.
    #[arg(short, long)]
    pub l1_rpc_endpoint: L1RpcEndpoint,

    /// Proposer gRPC endpoint.
    #[arg(short, long)]
    pub proposer_endpoint: GrpcUri,

    /// Sp1 cluster endpoint
    #[arg(short, long)]
    pub sp1_cluster_endpoint: Url,

    /// Run in mock mode?
    #[arg(long)]
    pub mock: bool,
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    println!("Starting Proposer service test...");
    color_eyre::install()?;

    // Initialize the tracing
    prover_logger::tracing(&Log::default());

    let cli = Cli::parse();

    // Setup the l1 rpc client
    let client = prover_alloy::AlloyProvider::new(
        &cli.l1_rpc_endpoint.url,
        prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
        prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
    )?;
    let l1_rpc_client = Arc::new(client);

    info!("L1 RPC client initialized");

    // Setup the proposer service
    let propser_service_config = ProposerServiceConfig {
        mock: cli.mock,
        client: ProposerClientConfig {
            proposer_endpoint: cli.proposer_endpoint,
            sp1_cluster_endpoint: cli.sp1_cluster_endpoint,
            request_timeout: proposer_client::config::default_request_timeout(),
            proving_timeout: proposer_client::config::default_proving_timeout(),
        },
        l1_rpc_endpoint: cli.l1_rpc_endpoint,
    };
    let mut proposer_service = if cli.mock {
        tower::ServiceBuilder::new()
            .service(ProposerService::new_mock(&propser_service_config, l1_rpc_client).await?)
            .boxed_clone()
    } else {
        tower::ServiceBuilder::new()
            .service(ProposerService::new_network(&propser_service_config, l1_rpc_client).await?)
            .boxed_clone()
    };
    info!("ProposerService initialized");

    // Perform proving request based on the input arguments
    let request = FepProposerRequest {
        last_proven_block: cli.last_proven_block,
        requested_end_block: cli.requested_end_block,
        l1_block_hash: B256::from_str(&cli.l1_block_hash)?,
    };
    match proposer_service.call(request).await {
        Ok(response) => {
            println!("Proposer response: {response:?}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e:?}");
            eyre::bail!(e);
        }
    }
}
