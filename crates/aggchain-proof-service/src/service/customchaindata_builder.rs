use std::{
    sync::Arc,
    task::{Context, Poll},
};

use futures::{future::BoxFuture, FutureExt as _};

use crate::context::AGGCHAIN_PROOF_PROGRAM_VERSION;

const AGGCHAIN_TYPE: u16 = 0x0001;
const AGGCHAIN_VKEY_SELECTOR: [u8; 4] =
    calculate_aggchain_vkey_selector(AGGCHAIN_PROOF_PROGRAM_VERSION, AGGCHAIN_TYPE);

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {}

#[derive(Debug)]
pub(crate) struct CustomChainDataBuilderRequest {}

#[derive(Debug, PartialEq)]
pub(crate) struct CustomChainDataBuilderResponse {}

pub(crate) struct CustomChainDataBuilderService<L1Rpc> {
    l1_rpc: Arc<L1Rpc>,
    network_id: u32,
}

impl Clone for CustomChainDataBuilderService<L1Rpc> {
    fn clone(&self) -> Self {
        CustomChainDataBuilderService {
            l1_rpc: self.l1_rpc.clone(),
            network_id: self.network_id,
        }
    }
}

impl<L1Rpc> CustomChainDataBuilderService<L1Rpc> {
    pub fn new(l1_rpc: Arc<L1Rpc>, network_id: u32) -> Self {
        CustomChainDataBuilderService { l1_rpc, network_id }
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

    fn call(&mut self, _req: CustomChainDataBuilderRequest) -> Self::Future {
        let rpc = self.l1_rpc.clone();

        async move {
            let _ = rpc.resolve_vkey(AGGCHAIN_VKEY_SELECTOR).await;

            Ok(CustomChainDataBuilderResponse {})
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

const fn calculate_aggchain_vkey_selector(program: u16, aggchain_type: u16) -> [u8; 4] {
    ((program as u32) << 16 | aggchain_type as u32).to_be_bytes()
}

#[cfg(test)]
mod tests {
    use futures::FutureExt as _;
    use mockall::predicate::eq;
    use prover_alloy::consensus::proofs::calculate_ommers_root;
    use tower::Service as _;

    use super::*;

    // Constants for default programs for ECDSA
    const ECDSA_DEFAULT: u16 = 0x00;
    // Constants for default programs for FEP
    const FEP_DEFAULT: u16 = 0x01;
    // Constant for custom FEP program
    const CUSTOM_FEP_PROGRAM: u16 = 0x02;
    // Constant for custom FEP program
    const CUSTOM_FEP_PROGRAM2: u16 = 0x03;

    // Aggchain type for
    const AGGCHAIN_TYPE_ECDSA: u16 = 0x00;

    const AGGCHAIN_TYPE_FEP: u16 = 0x01;

    #[test]
    fn aggchain_pattern() {
        // aggchain is using aggchain-type 0 and use the default ECDSA program
        assert_eq!(
            0b0000_0000_0000_0000_0000_0000_0000_0000,
            ((ECDSA_DEFAULT as u32) << 16 | AGGCHAIN_TYPE_ECDSA as u32)
        );
        // aggchain is using aggchain-type 0 and use the default FEP program -> Should
        // fail
        assert_eq!(
            0b0000_0000_0000_0001_0000_0000_0000_0000,
            ((FEP_DEFAULT as u32) << 16 | AGGCHAIN_TYPE_ECDSA as u32)
        );
        // aggchain is using aggchain-type 1 and use the default FEP program
        assert_eq!(
            0b0000_0000_0000_0001_0000_0000_0000_0001,
            ((FEP_DEFAULT as u32) << 16 | AGGCHAIN_TYPE_FEP as u32)
        );
        // aggchain is using aggchain-type 1 and use the default ECDSA program ->
        // Allowed as we'll support ECDSA for type 1
        assert_eq!(
            0b0000_0000_0000_0000_0000_0000_0000_0001,
            ((ECDSA_DEFAULT as u32) << 16 | AGGCHAIN_TYPE_FEP as u32)
        );
        // aggchain is using aggchain-type 1 and use its own FEP program
        assert_eq!(
            0b0000_0000_0000_0010_0000_0000_0000_0001,
            ((CUSTOM_FEP_PROGRAM as u32) << 16 | AGGCHAIN_TYPE_FEP as u32)
        );
        // aggchain is using aggchain-type 1 and use its own FEP program
        assert_eq!(
            0b0000_0000_0000_0011_0000_0000_0000_0001,
            ((CUSTOM_FEP_PROGRAM2 as u32) << 16 | AGGCHAIN_TYPE_FEP as u32)
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

        let mut service = CustomChainDataBuilderService::new(Arc::new(l1_rpc), 1);

        let request = CustomChainDataBuilderRequest {};
        let response = service.call(request).await.unwrap();

        assert_eq!(response, CustomChainDataBuilderResponse {});
    }
}
