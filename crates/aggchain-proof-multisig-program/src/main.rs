#![no_main]
sp1_zkvm::entrypoint!(main);

use aggchain_proof_multisig_core::AggchainProofMultisigWitness;

pub fn main() {
    let multisig_witness: AggchainProofMultisigWitness =
        sp1_zkvm::io::read::<AggchainProofMultisigWitness>();

    println!("cycle-tracker-report-start: verification");
    let aggchain_proof_public_values = multisig_witness.verify_multisig().unwrap();
    println!("cycle-tracker-report-end: verification");

    sp1_zkvm::io::commit(&aggchain_proof_public_values);
}
