use std::str::FromStr;
use std::sync::Arc;

use alloy_primitives::B256;
use anyhow::anyhow;
use clap::Parser;
use proposer_client::config::ProposerClientConfig;
use proposer_client::FepProposerRequest;
use proposer_service::config::ProposerServiceConfig;
use proposer_service::ProposerService;
use prover_alloy::L1RpcEndpoint;
use prover_logger::log::Log;
use sp1_sdk::SP1ProofWithPublicValues;
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

    /// Proposer JSON rpc endpoint.
    #[arg(short, long)]
    pub proposer_endpoint: Url,

    /// Sp1 cluster endpoint
    #[arg(short, long)]
    pub sp1_cluster_endpoint: Url,

    /// Run in mock mode?
    #[arg(long)]
    pub mock: bool,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let proof_response = "AAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFDQhMFYjcwCVBIgAFohyQCCWmAB4VpZwGHU3Y0zDIdQ16i5CHYfpcDJCRQDkiaoBphubtto591AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMB4668iyT3Fh6Y4wkvQhSzlMS58Dk+i14g0mNLZyQZC2dVUFe0tpJPa1haRUrmobHQG7brW8m1Hy7dwY5gXN1b2xHVSaI6m6/vddyAIfglL+P5JN+6vSlzWHuW94XdfsZsAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKtjx0yvzdmrtg2RMF8alNxn4CUwNOZVRq+jyZl07JZjWA2d3YDaw2LEnIOq3dbZRxyUeY6JJy4T2PrHCBBiyTpwAAAAAAAAAC3Y0LjAuMC1yYy4z";
    println!("trying to deserialize proof response {proof_response:?}");
    use anyhow::Context;
    use base64::Engine;
    let proof_bytes = base64::prelude::BASE64_STANDARD
        .decode(proof_response)
        .with_context(|| format!("deserializing base64"))?;
    let proof: SP1ProofWithPublicValues =
        bincode::deserialize(&proof_bytes).with_context(|| format!("deserializing proof"))?;
    println!("done!");
    if 2 * 2 == 4 {
        return Ok(());
    }
    // TODO: remove all that
    println!("Starting Proposer service test...");

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
            .service(ProposerService::new_mock(
                &propser_service_config,
                l1_rpc_client,
            )?)
            .boxed_clone()
    } else {
        tower::ServiceBuilder::new()
            .service(ProposerService::new_network(
                &propser_service_config,
                l1_rpc_client,
            )?)
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
            println!("Proposer response: {:?}", response);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(anyhow!(e.to_string()))
        }
    }
}
