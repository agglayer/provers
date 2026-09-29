use std::{
    borrow::Borrow,
    collections::HashMap,
    future::Future,
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
use tracing::info;

use crate::aggregation_prover::AggregationProver;

/// The SP1 network prover, along with a client for the network's program
/// registry, which holds the verifying key of every program proven on it.
pub struct NetworkAggregationProver {
    prover: NetworkProver,
    network: NetworkClient,
    rpc_url: String,
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
        info!(rpc_url = %self.rpc_url, %request_id, "Waiting for proof from Succinct network");
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
        fetch_aggregation_vkey(vk_hash, &self.vkeys, async {
            info!(rpc_url = %self.rpc_url, %vk_hash, "Fetching verification key from Succinct network");
            sp1_async(AssertUnwindSafe(self.network.get_program(vk_hash)))
                .await?
                .map_err(|error| eyre!(error))
        })
        .await
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

async fn fetch_aggregation_vkey(
    vk_hash: B256,
    vkeys: &Mutex<HashMap<B256, SP1VerifyingKey>>,
    fetch_program: impl Future<Output = eyre::Result<Option<GetProgramResponse>>>,
) -> eyre::Result<SP1VerifyingKey> {
    let cached = vkeys
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&vk_hash)
        .cloned();
    if let Some(vkey) = cached {
        return Ok(vkey);
    }

    let program = fetch_program
        .await
        .with_context(|| format!("Fetching program {vk_hash} from the SP1 network"))?;
    let vk_bytes = match program {
        Some(GetProgramResponse::Base(response)) => response.program.map(|program| program.vk),
        None => None,
        Some(_) => {
            return Err(eyre!(
                "Expected a Reserved registry response for program {vk_hash}"
            ));
        }
    }
    .ok_or_else(|| eyre!("Program {vk_hash} is not registered on the SP1 network"))?;

    // SP1's field deserializer can panic on noncanonical values.
    let (vkey, served_hash) = sp1_fast(|| -> eyre::Result<_> {
        let vkey: SP1VerifyingKey = agglayer_interop_types::bincode::sp1_compatible()
            .deserialize(&vk_bytes)
            .with_context(|| format!("Decoding the verifying key of program {vk_hash}"))?;

        let served_hash = NetworkClient::get_vk_hash(&vkey).map_err(|error| eyre!(error))?;
        Ok((vkey, served_hash))
    })??;
    if served_hash != vk_hash {
        return Err(eyre!(
            "The SP1 network served the verifying key of program {served_hash} for program \
             {vk_hash}"
        ));
    }

    vkeys
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(vk_hash, vkey.clone());

    Ok(vkey)
}

/// The vk hash, as the SP1 program registry keys it, of the program a
/// compressed proof proves: the digest committed in its recursion public
/// values.
fn proven_program_vk_hash(proof: &SP1ProofWithPublicValues) -> eyre::Result<B256> {
    let SP1Proof::Compressed(proof) = &proof.proof else {
        return Err(eyre!("The aggregation proof is not a compressed proof"));
    };
    let public_values = proof.proof.public_values.as_slice();
    if public_values.len() != RECURSIVE_PROOF_NUM_PV_ELTS {
        return Err(eyre!(
            "The aggregation proof carries {} public values, but a recursion proof requires \
             exactly {}",
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

    sp1_async(AssertUnwindSafe(async move {
        let prover = sp1_sdk::ProverClient::builder()
            .network()
            .rpc_url(&endpoint)
            .private_key(&private_key)
            .build()
            .await;
        let network = NetworkClient::new(signer, endpoint.clone(), prover.network_mode());

        NetworkAggregationProver {
            prover,
            network,
            rpc_url: endpoint,
            vkeys: Mutex::default(),
        }
    }))
    .await
}

#[cfg(test)]
mod tests {
    use alloy_primitives::b256;
    use sp1_sdk::{
        network::{get_default_rpc_url_for_mode, proto::base_types, NetworkMode},
        HashableKey as _, SP1ProofMode, SP1PublicValues, SP1_CIRCUIT_VERSION,
    };

    use super::*;

    // Use a registered key independent of this branch's rebuilt ELF. Generated
    // with SP1 6.2.2 from the aggchain-proof-builder ELF at
    // 5c51190e0c0edd1ee9ba8bc4383bd74f361760e7. ELF SHA256:
    // aef29d7db716240d516f8f8a2edbf539208764f4b03e189f9a75c9c5121828c6.
    const AGGCHAIN_VKEY_BYTES: &[u8] = include_bytes!("tests/aggchain_vkey_sp1_6_2_2.bin");
    const AGGCHAIN_VKEY_HASH: B256 =
        b256!("679bc13716cdb49416a9ca9e297b10d76390df2c343690d4172676c207517915");

    fn fixture_key() -> SP1VerifyingKey {
        agglayer_interop_types::bincode::sp1_compatible()
            .deserialize(AGGCHAIN_VKEY_BYTES)
            .expect("valid aggchain key fixture")
    }

    fn base_response(bytes: &[u8]) -> GetProgramResponse {
        GetProgramResponse::Base(base_types::GetProgramResponse {
            program: Some(base_types::Program {
                vk: bytes.to_vec(),
                ..Default::default()
            }),
        })
    }

    fn compressed_proof(public_values: Vec<SP1Field>) -> SP1ProofWithPublicValues {
        let mut proof = SP1ProofWithPublicValues::create_mock_proof(
            &fixture_key(),
            SP1PublicValues::new(),
            SP1ProofMode::Compressed,
            SP1_CIRCUIT_VERSION,
        );
        let SP1Proof::Compressed(compressed) = &mut proof.proof else {
            panic!("the mock proof is compressed");
        };
        compressed.proof.public_values = public_values;
        proof
    }

    #[test]
    fn reads_the_program_registry_hash_from_recursion_public_values() {
        let public_values = RecursionPublicValues::<SP1Field> {
            sp1_vk_digest: fixture_key().hash_koalabear(),
            ..Default::default()
        };
        let proof = compressed_proof(public_values.as_array().to_vec());

        assert_eq!(
            proven_program_vk_hash(&proof).expect("reading the key hash"),
            AGGCHAIN_VKEY_HASH,
        );
    }

    #[test]
    fn rejects_non_compressed_proofs() {
        let proof = SP1ProofWithPublicValues::create_mock_proof(
            &fixture_key(),
            SP1PublicValues::new(),
            SP1ProofMode::Core,
            SP1_CIRCUIT_VERSION,
        );

        let error = proven_program_vk_hash(&proof).expect_err("core proof must fail");
        assert_eq!(
            error.to_string(),
            "The aggregation proof is not a compressed proof"
        );
    }

    #[test]
    fn rejects_incorrect_recursion_public_value_lengths() {
        for length in [
            0,
            RECURSIVE_PROOF_NUM_PV_ELTS - 1,
            RECURSIVE_PROOF_NUM_PV_ELTS + 1,
        ] {
            let proof = compressed_proof(vec![SP1Field::default(); length]);

            let error = proven_program_vk_hash(&proof).expect_err("incorrect length must fail");
            assert_eq!(
                error.to_string(),
                format!(
                    "The aggregation proof carries {length} public values, but a recursion proof \
                     requires exactly {RECURSIVE_PROOF_NUM_PV_ELTS}"
                ),
            );
        }
    }

    #[tokio::test]
    async fn decodes_reserved_registry_response() -> eyre::Result<()> {
        let cached = Mutex::default();
        let actual = fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async {
            Ok(Some(base_response(AGGCHAIN_VKEY_BYTES)))
        })
        .await?;

        assert_eq!(
            agglayer_interop_types::bincode::sp1_compatible().serialize(&actual)?,
            AGGCHAIN_VKEY_BYTES,
        );
        assert_eq!(actual.hash_bytes(), AGGCHAIN_VKEY_HASH.0);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_missing_registry_programs() {
        for response in [None, Some(GetProgramResponse::Base(Default::default()))] {
            let cached = Mutex::default();
            let error = fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async { Ok(response) })
                .await
                .err()
                .expect("missing program must fail");

            assert_eq!(
                error.to_string(),
                format!("Program {AGGCHAIN_VKEY_HASH} is not registered on the SP1 network"),
            );
            assert!(cached.lock().expect("cache lock").is_empty());
        }
    }

    #[tokio::test]
    async fn rejects_malformed_registry_keys() {
        let cached = Mutex::default();
        let error = fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async {
            Ok(Some(base_response(&[0xff])))
        })
        .await
        .err()
        .expect("malformed key must fail");

        assert_eq!(
            error.to_string(),
            format!("Decoding the verifying key of program {AGGCHAIN_VKEY_HASH}"),
        );
        assert!(cached.lock().expect("cache lock").is_empty());
    }

    #[tokio::test]
    async fn rejects_noncanonical_registry_keys() {
        let bytes = vec![0xff; AGGCHAIN_VKEY_BYTES.len()];
        let cached = Mutex::default();
        let result = fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async {
            Ok(Some(base_response(&bytes)))
        })
        .await;

        assert!(result.is_err());
        assert!(cached.lock().expect("cache lock").is_empty());
    }

    #[tokio::test]
    async fn rejects_mismatched_registry_keys() {
        let requested_hash = B256::ZERO;
        let cached = Mutex::default();
        let error = fetch_aggregation_vkey(requested_hash, &cached, async {
            Ok(Some(base_response(AGGCHAIN_VKEY_BYTES)))
        })
        .await
        .err()
        .expect("mismatched key must fail");

        assert_eq!(
            error.to_string(),
            format!(
                "The SP1 network served the verifying key of program {AGGCHAIN_VKEY_HASH} for \
                 program {requested_hash}"
            ),
        );
        assert!(cached.lock().expect("cache lock").is_empty());
    }

    #[tokio::test]
    async fn propagates_registry_errors() {
        let cached = Mutex::default();
        let error = fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async {
            Err(eyre!("registry unavailable"))
        })
        .await
        .err()
        .expect("network failure must propagate");

        assert_eq!(
            error.to_string(),
            format!("Fetching program {AGGCHAIN_VKEY_HASH} from the SP1 network"),
        );
        assert_eq!(error.root_cause().to_string(), "registry unavailable");
        assert!(cached.lock().expect("cache lock").is_empty());
    }

    #[tokio::test]
    async fn reuses_a_cached_registry_key() -> eyre::Result<()> {
        let cached = Mutex::default();
        fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async {
            Ok(Some(base_response(AGGCHAIN_VKEY_BYTES)))
        })
        .await?;

        let actual = fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async {
            panic!("a cached key must not trigger another registry request");
        })
        .await?;

        assert_eq!(
            agglayer_interop_types::bincode::sp1_compatible().serialize(&actual)?,
            AGGCHAIN_VKEY_BYTES,
        );
        Ok(())
    }

    #[test_log::test(tokio::test)]
    #[ignore = "requires access to the SP1 Reserved registry; run explicitly"]
    async fn reserved_registry_serves_the_aggchain_proof_key() -> eyre::Result<()> {
        // Transitive dependencies enable both rustls providers, so the test
        // must select one.
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            rustls::crypto::aws_lc_rs::default_provider()
                .install_default()
                .map_err(|_| eyre!("Installing the TLS crypto provider"))?;
        }

        let network_mode = NetworkMode::Reserved;
        let signer = NetworkSigner::local(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .map_err(|error| eyre!(error))?;
        let network = NetworkClient::new(
            signer,
            get_default_rpc_url_for_mode(network_mode),
            network_mode,
        );
        let cached = Mutex::default();
        let actual = tokio::time::timeout(
            Duration::from_secs(30),
            fetch_aggregation_vkey(AGGCHAIN_VKEY_HASH, &cached, async {
                network
                    .get_program(AGGCHAIN_VKEY_HASH)
                    .await
                    .map_err(|error| eyre!(error))
            }),
        )
        .await
        .with_context(|| {
            format!(
                "Reserved registry request for aggchain proof program {AGGCHAIN_VKEY_HASH} timed \
                 out"
            )
        })??;

        assert_eq!(
            agglayer_interop_types::bincode::sp1_compatible().serialize(&actual)?,
            AGGCHAIN_VKEY_BYTES,
        );
        assert_eq!(actual.hash_bytes(), AGGCHAIN_VKEY_HASH.0);
        Ok(())
    }
}
