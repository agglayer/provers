#![no_main]
sp1_zkvm::entrypoint!(main);

use aggchain_proof_core::bridge::BridgeConstraintsInput;

pub fn main() {
    // Read the bridge constraints input from stdin
    let bridge_input: BridgeConstraintsInput = sp1_zkvm::io::read::<BridgeConstraintsInput>();
    
    // Verify the bridge constraints - this will panic if verification fails
    bridge_input.verify().unwrap();
    
    // Commit the result to indicate successful verification
    sp1_zkvm::io::commit(&true);
} 