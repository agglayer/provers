#![no_main]
sp1_zkvm::entrypoint!(main);

use aggchain_proof_core::proof::AggchainProofWitness;

// Mock counterpart of `aggchain-proof-program`: it reads the same witness and
// commits the same public values, but skips the FEP / ECDSA and bridge
// constraint verification entirely, so any witness is accepted.
pub fn main() {
    let aggchain_witness: AggchainProofWitness = sp1_zkvm::io::read::<AggchainProofWitness>();

    sp1_zkvm::io::commit(&aggchain_witness.public_values());
}
