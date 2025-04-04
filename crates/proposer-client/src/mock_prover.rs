use std::time::Duration;

use alloy_primitives::B256;
use anyhow::Context;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use bincode::Options as _;
use jsonrpsee::{
    core::client::{ClientT, Error as JsonRpcError},
    http_client::HttpClient,
    rpc_params,
};
use sp1_sdk::{
    CpuProver, Prover as _, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerificationError,
    SP1VerifyingKey,
};
use tracing::error;
use url::Url;

use crate::aggregation_prover::AggregationProver;

pub struct MockProver {
    rpc_url: Url,
    sp1_prover: CpuProver,
}

impl MockProver {
    pub fn new(rpc_url: Url) -> anyhow::Result<MockProver> {
        Ok(MockProver {
            rpc_url,
            sp1_prover: sp1_sdk::CpuProver::new(),
        })
    }
}

#[tonic::async_trait]
impl AggregationProver for MockProver {
    fn compute_pkey_vkey(&self, program: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.sp1_prover.setup(program)
    }

    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> anyhow::Result<SP1ProofWithPublicValues> {
        let real_request_id: u64 = u64::from_be_bytes(request_id[..8].try_into().unwrap());

        let mut builder = HttpClient::builder();
        if let Some(timeout) = timeout {
            builder = builder.request_timeout(timeout);
        }
        let client = builder
            .build(&self.rpc_url)
            .with_context(|| format!("building RPC client for {:?}", self.rpc_url))?;

        let proof_response: String = loop {
            match client
                .request("proofs_getAggProof", rpc_params![real_request_id])
                .await
            {
                Ok(proof_response) => break proof_response,
                Err(JsonRpcError::Call(error))
                    if error.code() == -32000
                        && error.message().contains("proof request not complete") =>
                {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
                Err(error) => {
                    error!(?error, "proofs_getAggProof failed");
                    return Err(anyhow::Error::from(error)
                        .context(format!("getting agg proof {request_id}")));
                }
            };
        };
        tracing::info!("got proof response {proof_response:?}");
        let proof_bytes = BASE64_STANDARD
            .decode(proof_response)
            .with_context(|| format!("deserializing base64 for proof {request_id}"))?;
        let proof = bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .deserialize(&proof_bytes)
            .with_context(|| format!("deserializing proof {request_id}"))?;

        Ok(proof)
    }

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError> {
        self.sp1_prover.verify(proof, vkey)
    }
}
