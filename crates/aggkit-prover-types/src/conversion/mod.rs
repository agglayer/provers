use agglayer_interop::{grpc as legacy_grpc, types::Digest};
use agglayer_interop_grpc_types as grpc;
use prost::bytes::Bytes;
use prost_legacy::Message as _;

pub mod v1;

/// Pairs a prost 0.14 interop message with the prost 0.13 message of
/// `agglayer-interop` 0.18 generated from the same protobuf definition.
trait LegacyCounterpart: prost::Message {
    type Legacy: prost_legacy::Message + Default;
}

impl LegacyCounterpart for grpc::v1::FixedBytes32 {
    type Legacy = legacy_grpc::v1::FixedBytes32;
}

impl LegacyCounterpart for grpc::v1::L1InfoTreeLeafWithContext {
    type Legacy = legacy_grpc::v1::L1InfoTreeLeafWithContext;
}

impl LegacyCounterpart for grpc::v1::MerkleProof {
    type Legacy = legacy_grpc::v1::MerkleProof;
}

/// Conversion of an interop gRPC message into its interop type.
// tonic 0.14 requires prost 0.14 messages, which come from
// https://github.com/agglayer/interop/pull/296. The validating conversions to
// the interop types are however only implemented by `agglayer-interop` 0.18,
// for its own prost 0.13 messages, and the interop types cannot be upgraded
// without changing the proof program dependencies. Both message sets share
// their wire format, so the message is re-encoded to reuse those conversions.
// This goes away once the workspace can depend on an interop release that
// ships prost 0.14.
pub(crate) trait TryIntoInterop<Target> {
    fn try_into_interop(self) -> eyre::Result<Target>;
}

impl<Message, Target> TryIntoInterop<Target> for Message
where
    Message: LegacyCounterpart,
    Message::Legacy: TryInto<Target>,
    <Message::Legacy as TryInto<Target>>::Error: std::error::Error + Send + Sync + 'static,
{
    fn try_into_interop(self) -> eyre::Result<Target> {
        let legacy = Message::Legacy::decode(self.encode_to_vec().as_slice())?;

        Ok(legacy.try_into()?)
    }
}

pub fn digest_to_grpc(digest: Digest) -> grpc::v1::FixedBytes32 {
    grpc::v1::FixedBytes32 {
        value: Bytes::copy_from_slice(digest.as_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use agglayer_interop::types::{Digest, MerkleProof};

    use super::*;

    #[test]
    fn digest_round_trips_through_grpc() -> eyre::Result<()> {
        let digest = Digest([0x42; 32]);

        let converted: Digest = digest_to_grpc(digest).try_into_interop()?;

        assert_eq!(converted, digest);
        Ok(())
    }

    #[test]
    fn fixed_bytes_of_wrong_length_are_rejected() {
        let too_short = grpc::v1::FixedBytes32 {
            value: Bytes::from_static(&[0x42; 31]),
        };

        let converted: eyre::Result<Digest> = too_short.try_into_interop();

        assert!(converted.is_err());
    }

    #[test]
    fn merkle_proof_keeps_root_and_siblings() -> eyre::Result<()> {
        let root = Digest([0x01; 32]);
        let siblings: [Digest; 32] = std::array::from_fn(|index| Digest([index as u8; 32]));
        let merkle_proof = grpc::v1::MerkleProof {
            root: Some(digest_to_grpc(root)),
            siblings: siblings.iter().copied().map(digest_to_grpc).collect(),
        };

        let converted: MerkleProof = merkle_proof.try_into_interop()?;

        assert_eq!(converted, MerkleProof::new(root, siblings));
        Ok(())
    }
}
