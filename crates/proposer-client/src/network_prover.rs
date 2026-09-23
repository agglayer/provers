use std::{
    borrow::Borrow,
    collections::HashMap,
    panic::AssertUnwindSafe,
    sync::{Mutex, PoisonError},
    time::Duration,
};

use alloy_primitives::B256;
use eyre::{eyre, Context};
use prover_executor::{sp1_async, sp1_fast};
use slop_algebra::PrimeField32;
use sp1_primitives::SP1Field;
use sp1_recursion_executor::{RecursionPublicValues, RECURSIVE_PROOF_NUM_PV_ELTS};
use sp1_sdk::{
    network::{proto::GetProgramResponse, signer::NetworkSigner, NetworkClient},
    NetworkProver, Prover, ProvingKey, SP1Proof, SP1ProofWithPublicValues, SP1ProvingKey,
    SP1VerifyingKey,
};

use crate::aggregation_prover::AggregationProver;

/// The SP1 network prover, along with a client for the network's program
/// registry, which holds the verifying key of every program proven on it.
pub struct NetworkAggregationProver {
    prover: NetworkProver,
    network: NetworkClient,
    /// Verifying keys already fetched from the program registry, by vk hash.
    vkeys: Mutex<HashMap<B256, SP1VerifyingKey>>,
}

#[tonic::async_trait]
impl AggregationProver for NetworkAggregationProver {
    async fn compute_pkey_vkey(
        &self,
        program: &[u8],
    ) -> eyre::Result<(SP1ProvingKey, SP1VerifyingKey)> {
        // TODO: Figure out a way to kill this struct if there's an unwind, and
        // start again with a fresh Prover
        let proving_key = sp1_async(AssertUnwindSafe(async {
            self.prover.setup(program.into()).await
        }))
        .await?
        .map_err(|error| eyre!(error.to_string()))?;
        let verifying_key = proving_key.verifying_key().clone();
        Ok((proving_key, verifying_key))
    }

    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> eyre::Result<SP1ProofWithPublicValues> {
        // TODO: Figure out a way to kill this struct if there's an unwind, and
        // start again with a fresh Prover
        sp1_async(AssertUnwindSafe(
            self.prover.wait_proof(request_id, timeout, None),
        ))
        .await?
        .map_err(|e| eyre!(e))
        .context("Failed waiting for proof")
    }

    async fn aggregation_vkey(
        &self,
        proof: &SP1ProofWithPublicValues,
    ) -> eyre::Result<SP1VerifyingKey> {
        let vk_hash = proven_program_vk_hash(proof)?;

        let cached = self
            .vkeys
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&vk_hash)
            .cloned();
        if let Some(vkey) = cached {
            return Ok(vkey);
        }

        let program = sp1_async(AssertUnwindSafe(self.network.get_program(vk_hash)))
            .await?
            .map_err(|error| eyre!(error))
            .with_context(|| format!("Fetching program {vk_hash} from the SP1 network"))?;
        let vk_bytes = match program {
            Some(GetProgramResponse::Auction(response)) => {
                response.program.map(|program| program.vk)
            }
            Some(GetProgramResponse::Base(response)) => response.program.map(|program| program.vk),
            None => None,
        }
        .ok_or_else(|| eyre!("Program {vk_hash} is not registered on the SP1 network"))?;

        let vkey: SP1VerifyingKey = agglayer_interop_types::bincode::sp1_compatible()
            .deserialize(&vk_bytes)
            .with_context(|| format!("Decoding the verifying key of program {vk_hash}"))?;

        let served_hash = NetworkClient::get_vk_hash(&vkey).map_err(|error| eyre!(error))?;
        if served_hash != vk_hash {
            return Err(eyre!(
                "The SP1 network served the verifying key of program {served_hash} for program \
                 {vk_hash}"
            ));
        }

        self.vkeys
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(vk_hash, vkey.clone());

        Ok(vkey)
    }

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> eyre::Result<()> {
        // TODO: Figure out a way to kill this struct if there's an unwind, and
        // start again with a fresh Prover
        sp1_fast(AssertUnwindSafe(|| self.prover.verify(proof, vkey, None)))?
            .map_err(|error| eyre!(error.to_string()))
    }
}

/// The vk hash, as the SP1 program registry keys it, of the program a
/// compressed proof proves: the digest committed in its recursion public
/// values.
fn proven_program_vk_hash(proof: &SP1ProofWithPublicValues) -> eyre::Result<B256> {
    let SP1Proof::Compressed(proof) = &proof.proof else {
        return Err(eyre!("The aggregation proof is not a compressed proof"));
    };
    let public_values = proof.proof.public_values.as_slice();
    if public_values.len() < RECURSIVE_PROOF_NUM_PV_ELTS {
        return Err(eyre!(
            "The aggregation proof carries {} public values, fewer than a recursion proof's {}",
            public_values.len(),
            RECURSIVE_PROOF_NUM_PV_ELTS
        ));
    }
    let public_values: &RecursionPublicValues<SP1Field> = public_values.borrow();

    let words = public_values
        .sp1_vk_digest
        .map(|word| word.as_canonical_u32().to_be_bytes());
    Ok(B256::try_from(words.as_flattened())?)
}

pub async fn new_network_prover<T: AsRef<str>>(
    endpoint: T,
) -> eyre::Result<NetworkAggregationProver> {
    let endpoint = endpoint.as_ref().to_string();
    let private_key = std::env::var("NETWORK_PRIVATE_KEY").context(
        "Failed to get NETWORK_PRIVATE_KEY, when building NetworkProver for proposer-client",
    )?;
    let signer = NetworkSigner::local(&private_key)
        .map_err(|error| eyre!(error.to_string()))
        .context("Creating the SP1 network signer")?;

    let prover = sp1_async(AssertUnwindSafe({
        let endpoint = endpoint.clone();
        async move {
            sp1_sdk::ProverClient::builder()
                .network()
                .rpc_url(&endpoint)
                .private_key(&private_key)
                .build()
                .await
        }
    }))
    .await?;
    let network = NetworkClient::new(signer, endpoint, prover.network_mode());

    Ok(NetworkAggregationProver {
        prover,
        network,
        vkeys: Mutex::default(),
    })
}

#[cfg(test)]
mod tests {
    use sp1_sdk::{HashableKey as _, SP1ProofMode, SP1PublicValues, SP1_CIRCUIT_VERSION};

    use super::*;

    const ELF: &[u8] =
        include_bytes!("../../aggchain-proof-builder/elf/riscv64im-succinct-zkvm-elf");

    async fn vkey() -> SP1VerifyingKey {
        let prover = sp1_sdk::ProverClient::builder().mock().build().await;
        let proving_key = prover
            .setup(ELF.into())
            .await
            .expect("setting up the program");
        proving_key.verifying_key().clone()
    }

    fn compressed_proof(vkey: &SP1VerifyingKey) -> SP1ProofWithPublicValues {
        SP1ProofWithPublicValues::create_mock_proof(
            vkey,
            SP1PublicValues::new(),
            SP1ProofMode::Compressed,
            SP1_CIRCUIT_VERSION,
        )
    }

    #[tokio::test]
    async fn reads_the_vk_hash_the_program_registry_keys_by() {
        let vkey = vkey().await;
        let mut proof = compressed_proof(&vkey);
        let SP1Proof::Compressed(compressed) = &mut proof.proof else {
            panic!("the mock proof is compressed");
        };
        let public_values = RecursionPublicValues::<SP1Field> {
            sp1_vk_digest: vkey.hash_koalabear(),
            ..Default::default()
        };
        compressed.proof.public_values = public_values.as_array().to_vec();

        assert_eq!(
            proven_program_vk_hash(&proof).expect("reading the vk hash"),
            NetworkClient::get_vk_hash(&vkey).expect("hashing the vkey"),
        );
    }

    #[tokio::test]
    async fn rejects_a_proof_without_recursion_public_values() {
        let proof = compressed_proof(&vkey().await);

        assert!(proven_program_vk_hash(&proof).is_err());
    }
}
