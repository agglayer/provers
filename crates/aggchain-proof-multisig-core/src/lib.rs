use std::collections::BTreeMap;

use agglayer_primitives::{keccak::keccak256_combine, Digest};
pub use alloy_primitives::{Address, Signature};
use serde::{Deserialize, Serialize};
use unified_bridge::AggchainProofPublicValues;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AggchainProofMultisigWitness {
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// New local exit root.
    pub new_local_exit_root: Digest,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Origin network for which the proof was generated.
    pub origin_network: u32,
    /// Commitment on the imported bridge exits.
    pub commit_imported_bridge_exits: Digest,
    /// Chain-specific commitment.
    pub chain_specific_commitment: Digest,
    /// Signatures with their addresses.
    pub signatures_with_pk: BTreeMap<Address, Signature>,
}

#[inline]
fn verify_signature(expected: Address, signature: Signature, message: Digest) -> Result<(), ()> {
    let recovered = signature
        .recover_address_from_prehash((&message.0).into())
        .map_err(|_| ())?;

    (recovered == expected).then_some(()).ok_or(())
}

impl AggchainProofMultisigWitness {
    #[inline]
    pub fn verify_multisig(&self) -> Result<AggchainProofPublicValues, ()> {
        let message = Digest::default();
        for (&address, &signature) in self.signatures_with_pk.iter() {
            verify_signature(address, signature, message)?;
        }

        Ok(self.public_values())
    }

    #[inline]
    pub fn public_values(&self) -> AggchainProofPublicValues {
        AggchainProofPublicValues {
            prev_local_exit_root: self.prev_local_exit_root,
            new_local_exit_root: self.new_local_exit_root,
            l1_info_root: self.l1_info_root,
            origin_network: self.origin_network.into(),
            commit_imported_bridge_exits: self.commit_imported_bridge_exits,
            aggchain_params: self.aggchain_params(),
        }
    }

    /// Definition of the aggchain params which is re-constructed in the L1
    #[inline]
    pub fn aggchain_params(&self) -> Digest {
        keccak256_combine([self.signer_hashchain(), self.chain_specific_commitment()])
    }

    /// Chain-specific commitment which specify block numbers among other things
    #[inline]
    pub fn chain_specific_commitment(&self) -> Digest {
        // Dummy for now, let's say that it is a bunch of keccak for now
        let nb_fields_to_keccak = 10;

        keccak256_combine(vec![self.chain_specific_commitment; nb_fields_to_keccak])
    }

    /// Hash chain on the public keys used to verify the signatures.
    #[inline]
    pub fn signer_hashchain(&self) -> Digest {
        // NOTE: An indexing of the used public keys will need to be bubble up
        // from the chain up to the L1. Can be a bitmap to explicit which public
        // keys from the L1 are used. or an explicit ordered list of
        // public keys or indexes for a map in L1.
        self.signatures_with_pk
            .keys()
            .map(|&signer| SignerHashchain::from(signer))
            .fold(Digest::default(), |acc, padded_signer| {
                keccak256_combine([acc, padded_signer.0]).into()
            })
    }
}

#[derive(Default, Debug)]
pub struct SignerHashchain(pub Digest);

impl AsRef<[u8]> for SignerHashchain {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0 .0
    }
}

impl From<Digest> for SignerHashchain {
    fn from(value: Digest) -> Self {
        Self(value)
    }
}

impl From<Address> for SignerHashchain {
    /// Each address in the L1 will be on 32bytes so the hash chain will need
    /// to be done out of 32bytes elements.
    #[inline]
    fn from(signer: Address) -> Self {
        let mut padded_signer = [0; 32];
        padded_signer[12..32].copy_from_slice(signer.0.as_slice());
        Digest::from(padded_signer).into()
    }
}
