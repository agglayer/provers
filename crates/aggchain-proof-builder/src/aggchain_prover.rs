use std::sync::Arc;
use std::time::Duration;

use alloy::primitives::B256;
use anyhow::Error;
use sp1_sdk::{NetworkProver, Prover, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerifyingKey};

/// This prover waits for the SP1 cluster generated
/// AggSpanProof based on the proof id.
#[tonic::async_trait]
pub trait AggChainProver {
    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> Result<SP1ProofWithPublicValues, anyhow::Error>;

    fn get_vkey(&self, elf: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey);
}

#[derive(Clone)]
pub struct AggChainNetworkProver {
    client: Arc<NetworkProver>,
}

impl AggChainNetworkProver {
    pub fn new(endpoint: &str) -> AggChainNetworkProver {
        AggChainNetworkProver {
            client: Arc::new(
                sp1_sdk::ProverClient::builder()
                    .network()
                    .rpc_url(endpoint)
                    .build(),
            ),
        }
    }
}

#[tonic::async_trait]
impl AggChainProver for AggChainNetworkProver {
    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> Result<SP1ProofWithPublicValues, Error> {
        self.client.wait_proof(request_id, timeout).await
    }

    fn get_vkey(&self, elf: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.client.setup(elf)
    }
}
