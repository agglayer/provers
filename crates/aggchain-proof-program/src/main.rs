#![no_main]
sp1_zkvm::entrypoint!(main);

use aggchain_proof_core::proof::AggchainProofWitness;

pub fn main() {
    let aggchain_witness: AggchainProofWitness = sp1_zkvm::io::read::<AggchainProofWitness>();

    let aggchain_proof_public_values = aggchain_witness.verify_aggchain_inputs().unwrap();

    sp1_zkvm::io::commit(&aggchain_proof_public_values);
}
