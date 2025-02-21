use std::{
    sync::Arc,
    task::{Context, Poll},
};

use futures::{future::BoxFuture, FutureExt as _};
use tracing::error;

use crate::context::AGGCHAIN_PROOF_PROGRAM_VERSION;

const AGGCHAIN_TYPE: u16 = 0x0001;
const AGGCHAIN_VKEY_SELECTOR: [u8; 4] =
    calculate_aggchain_vkey_selector(AGGCHAIN_PROOF_PROGRAM_VERSION, AGGCHAIN_TYPE);

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[derive(Debug)]
pub(crate) struct CustomChainDataBuilderRequest {
    pub(crate) l2_block_number: u64,
    pub(crate) output_root: [u8; 32],
}

#[derive(Debug, PartialEq)]
pub(crate) struct CustomChainDataBuilderResponse {
    pub custom_chain_data: [u8; 66],
}

pub(crate) struct CustomChainDataBuilderService<L1Rpc> {
    l1_rpc: Arc<L1Rpc>,
    network_id: u32,
    vkey: Arc<[u32; 8]>,
}

impl Clone for CustomChainDataBuilderService<L1Rpc> {
    fn clone(&self) -> Self {
        CustomChainDataBuilderService {
            l1_rpc: self.l1_rpc.clone(),
            network_id: self.network_id,
            vkey: self.vkey.clone(),
        }
    }
}

impl<L1Rpc> CustomChainDataBuilderService<L1Rpc> {
    pub fn new(l1_rpc: Arc<L1Rpc>, network_id: u32, vkey: [u32; 8]) -> Self {
        CustomChainDataBuilderService {
            l1_rpc,
            network_id,
            vkey: Arc::new(vkey),
        }
    }
}

impl<L1Rpc> tower::Service<CustomChainDataBuilderRequest> for CustomChainDataBuilderService<L1Rpc>
where
    L1Rpc: AggchainVkeyResolver + Send + Sync + 'static,
{
    type Response = CustomChainDataBuilderResponse;
    type Error = Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: CustomChainDataBuilderRequest) -> Self::Future {
        let rpc = self.l1_rpc.clone();

        let vkey = self.vkey.clone();
        async move {
            let registered_vkey = rpc.resolve_vkey(AGGCHAIN_VKEY_SELECTOR).await?;
            if *vkey != registered_vkey {
                error!(
                    "L1 registered vKey doesn't match the one configured in the \
                     aggchain-proof-service"
                )
            }

            Ok(CustomChainDataBuilderResponse {
                custom_chain_data: calculate_custom_chain_data(
                    AGGCHAIN_PROOF_PROGRAM_VERSION,
                    req.output_root,
                    req.l2_block_number,
                ),
            })
        }
        .boxed()
    }
}

#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock)]
trait AggchainVkeyResolver {
    async fn resolve_vkey(&self, aggchain_vkey_selector: [u8; 4]) -> Result<[u32; 8], Error>;
}

pub struct L1Rpc {}

#[async_trait::async_trait]
impl AggchainVkeyResolver for L1Rpc {
    async fn resolve_vkey(&self, _aggchain_vkey_selector: [u8; 4]) -> Result<[u32; 8], Error> {
        // Implementation for resolving vkey from L1 RPC
        unimplemented!()
    }
}

pub fn calculate_custom_chain_data(
    selector: u16,
    output_root: [u8; 32],
    l2_block_number: u64,
) -> [u8; 66] {
    let mut custom_chain_data = [0u8; 66];

    // Convert values to bytes in big-endian order
    custom_chain_data[0..2].copy_from_slice(&selector.to_be_bytes());
    custom_chain_data[2..34].copy_from_slice(&output_root);
    custom_chain_data[58..66].copy_from_slice(&l2_block_number.to_be_bytes());

    custom_chain_data
}

pub const fn calculate_aggchain_vkey_selector(program: u16, aggchain_type: u16) -> [u8; 4] {
    (((program as u32) << 16) | aggchain_type as u32).to_be_bytes()
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::PathBuf};

    use futures::FutureExt as _;
    use mockall::predicate::eq;
    use serde::{Deserialize, Deserializer};
    use tower::Service as _;

    use super::*;

    // Constants for default programs for ECDSA
    const ECDSA_DEFAULT: u16 = 0x00;
    // Constants for default programs for FEP
    const FEP_DEFAULT: u16 = 0x00;
    // Constant for custom FEP program
    const CUSTOM_FEP_PROGRAM: u16 = 0x01;
    // Constant for custom FEP program
    const CUSTOM_FEP_PROGRAM2: u16 = 0x02;

    // Aggchain type for
    const AGGCHAIN_TYPE_ECDSA: u16 = 0x00;

    const AGGCHAIN_TYPE_FEP: u16 = 0x01;

    #[test]
    fn aggchain_pattern() {
        // aggchain is using aggchain-type 0 and use the default ECDSA program
        assert_eq!(
            0b0000_0000_0000_0000_0000_0000_0000_0000,
            (((ECDSA_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_ECDSA as u32)
        );
        // aggchain is using aggchain-type 0 and use the default FEP program -> Should
        // fail
        assert_eq!(
            0b0000_0000_0000_0000_0000_0000_0000_0000,
            (((FEP_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_ECDSA as u32)
        );
        // aggchain is using aggchain-type 1 and use the default FEP program
        assert_eq!(
            0b0000_0000_0000_0000_0000_0000_0000_0001,
            (((FEP_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
        );
        // aggchain is using aggchain-type 1 and use the default ECDSA program ->
        // Allowed as we'll support ECDSA for type 1
        assert_eq!(
            0b0000_0000_0000_0000_0000_0000_0000_0001,
            (((ECDSA_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
        );
        // aggchain is using aggchain-type 1 and use its own FEP program
        assert_eq!(
            0b0000_0000_0000_0001_0000_0000_0000_0001,
            (((CUSTOM_FEP_PROGRAM as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
        );
        // aggchain is using aggchain-type 1 and use its own FEP program
        assert_eq!(
            0b0000_0000_0000_0010_0000_0000_0000_0001,
            (((CUSTOM_FEP_PROGRAM2 as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
        );
    }

    #[tokio::test]
    async fn test_custom_chain_data_builder_service() {
        let mut l1_rpc = MockAggchainVkeyResolver::new();
        l1_rpc
            .expect_resolve_vkey()
            .with(eq(calculate_aggchain_vkey_selector(
                FEP_DEFAULT,
                AGGCHAIN_TYPE_FEP,
            )))
            .returning(|_| async { Ok([0u32; 8]) }.boxed());

        let mut service = CustomChainDataBuilderService::new(Arc::new(l1_rpc), 1, [0u32; 8]);

        let request = CustomChainDataBuilderRequest {
            l2_block_number: 10u64,
            output_root: [1u8; 32],
        };

        let response = service.call(request).await.unwrap();

        let mut expected = [0u8; 66];
        // program selector
        expected[0..2].copy_from_slice(&[0, 0]);

        // output root
        expected[2..34].copy_from_slice(&[1u8; 32]);
        // l2 block number
        expected[58..66].copy_from_slice(&10u64.to_be_bytes());

        assert_eq!(
            response,
            CustomChainDataBuilderResponse {
                custom_chain_data: expected
            }
        );
    }

    #[derive(Debug, Deserialize)]
    struct TestVectorEntry {
        input: TestVectorEntryInput,
        output: TestVectorEntryOutput,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestVectorEntryInput {
        #[serde(deserialize_with = "hex_to_u16")]
        aggchain_type: u16,
        #[serde(deserialize_with = "hex_to_u16")]
        aggchain_v_key_selector: u16,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestVectorEntryOutput {
        #[serde(deserialize_with = "hex_to_u32")]
        final_aggchain_v_key_selector: u32,
    }
    fn hex_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        u32::from_str_radix(s.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
    }
    fn hex_to_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        u16::from_str_radix(s.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
    }

    #[test]
    fn test_aggchain_selector() {
        let path: PathBuf = env!("CARGO_MANIFEST_DIR").parse().unwrap();

        let test_vectors = path.join("test-vectors/aggchain-selector.json");
        let file = File::open(test_vectors).expect("Failed to open file");

        let data: Vec<TestVectorEntry> =
            serde_json::from_reader(file).expect("Failed to parse JSON");

        for TestVectorEntry { input, output } in data {
            assert_eq!(
                calculate_aggchain_vkey_selector(
                    input.aggchain_v_key_selector,
                    input.aggchain_type
                ),
                output.final_aggchain_v_key_selector.to_be_bytes()
            );
        }
    }
}
