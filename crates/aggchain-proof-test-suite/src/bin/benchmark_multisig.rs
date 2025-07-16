use std::{collections::BTreeMap, time::Instant};

use aggchain_proof_multisig_core::AggchainProofMultisigWitness;
use agglayer_primitives::Digest;
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use alloy_primitives::{Address, Signature};
use sp1_sdk::{ProverClient, SP1Stdin};

/// ELF of the AP multisig program
pub const AP_MULTISIG_ELF: &[u8] =
    include_bytes!("../../../aggchain-proof-multisig-program/elf/aggchain-proof-multisig-program");

fn generate_signature(commitment: Digest) -> (Address, Signature) {
    let signer = PrivateKeySigner::random();
    let signature: Signature = signer.sign_hash_sync(&commitment.into()).unwrap();

    (signer.address(), signature)
}

/// Returns k signatures to be proven
pub fn generate_multisig_witness(nb_signatures: usize) -> SP1Stdin {
    let commitment = Digest::default();
    let signatures_with_pk: BTreeMap<Address, Signature> = (0..nb_signatures)
        .map(|_| generate_signature(commitment))
        .collect();

    let witness = AggchainProofMultisigWitness {
        signatures_with_pk,
        ..Default::default()
    };

    // for (address, signature) in witness.signatures_with_pk.iter() {
    //     println!("Address: 0x{}", address);
    //     println!("Signature: 0x{}", signature);
    // }

    let mut stdin = SP1Stdin::new();
    stdin.write(&witness);

    stdin
}

fn benchmark_multisig_cycles(start: usize, end: usize, step: usize) -> BTreeMap<usize, (u64, f64)> {
    std::env::set_var("SP1_PROVER", "cpu");
    let client = ProverClient::from_env();
    let (pk, _) = client.setup(AP_MULTISIG_ELF);
    let mut results = BTreeMap::new();

    for nb_signers in (start..=end).step_by(step) {
        // Generate witness
        let stdin = generate_multisig_witness(nb_signers);

        // Execute to get cycle numbers
        let (_, report) = client
            .execute(AP_MULTISIG_ELF, &stdin)
            .run()
            .expect("execution failed");

        let cycles = report.cycle_tracker.get("verification").unwrap();

        let start_time = Instant::now();
        let _ = client
            .prove(&pk, &stdin)
            .compressed()
            .run()
            .expect("proving failed");
        let proving_time = start_time.elapsed().as_secs_f64();

        results.insert(nb_signers, (*cycles, proving_time));
    }

    results
}

pub fn main() {
    let cycles_per_signatures = benchmark_multisig_cycles(3, 33, 6);

    for (nb_signers, (cycles, proving_time)) in cycles_per_signatures.iter() {
        println!("{nb_signers},{cycles},{proving_time}");
    }
}
