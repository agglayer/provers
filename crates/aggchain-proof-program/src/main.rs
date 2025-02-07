#![no_main]
sp1_zkvm::entrypoint!(main);

use aggchain_proof_core::proof::AggchainProofWitness;
use bincode::Options;

pub fn main() {
    let mut aggchain_witness: AggchainProofWitness = sp1_zkvm::io::read::<AggchainProofWitness>();

    let outputs = aggchain_witness.generate_aggchain_proof().unwrap();

    let aggchain_proof_public_inputs = bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&aggchain_proof_public_inputs);
}
