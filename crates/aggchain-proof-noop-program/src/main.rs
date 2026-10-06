#![no_main]
sp1_zkvm::entrypoint!(main);

use unified_bridge::AggchainProofPublicValues;

// Recovery counterpart of `aggchain-proof-program`: it commits the public
// values it is given without verifying anything. This is a real SP1 proof of an
// empty program, unrelated to the SP1 mock prover. The aggchain contract only
// accepts it under the noop selector, with its vkey registered per rollup in
// `ownedAggchainVKeys`, never on the `AgglayerGateway`.
pub fn main() {
    let public_values = sp1_zkvm::io::read::<AggchainProofPublicValues>();

    sp1_zkvm::io::commit(&public_values);
}
