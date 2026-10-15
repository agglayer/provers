//! Host-side check of the sp1-cc sketch builder against a Besu QBFT node.
//!
//! Uses the `eth_like` executor of the local sp1-cc fork (`EvmFlavor::EthLike`)
//! and a Paris-active sketch genesis
//! (`vectors/bridge/sketches/sketch-genesis.json`: the Besu config plus
//! `terminalTotalDifficulty: 0`).
//!
//! Ignored by default. Run with:
//!
//! ```text
//! BESU_RPC_URL=http://localhost:18541 \
//! BESU_GENESIS=<path>/vectors/bridge/sketches/sketch-genesis.json \
//! BESU_GER_ADDR=0xa40d5f56745a118d0906a34e69aec8c0db1cb8fa \
//! BESU_BRIDGE_ADDR=0x2a3dd3eb832af982ec71669e178424b10dca2ede \
//! BESU_BLOCKS=31,47 BESU_OUT_DIR=<path>/vectors/bridge/sketches \
//! cargo test -p aggchain-proof-contracts --test besu_sketch -- --ignored --nocapture
//! ```

use aggchain_proof_core::bridge::{BridgeL2SovereignChain, L2_GER_ADDR};
use agglayer_interop::types::bincode;
use alloy::{
    primitives::{keccak256, Address, B256},
    providers::{Provider, ProviderBuilder},
    sol,
};
use sp1_cc_client_executor::{ClientExecutor, ContractInput, Genesis};
use sp1_cc_host_executor::EvmSketch;

sol! {
    interface GerL2 {
        function insertedGERHashChain() external view returns (bytes32);
        function removedGERHashChain() external view returns (bytes32);
        function bridgeAddress() external view returns (address);
    }
}

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set"))
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn besu_sketches() {
    let rpc: url::Url = env("BESU_RPC_URL").parse().unwrap();
    let genesis_str = std::fs::read_to_string(env("BESU_GENESIS")).unwrap();
    let parsed: alloy::genesis::Genesis =
        serde_json::from_str(&genesis_str).expect("genesis parse");
    let genesis = Genesis::Custom(parsed.config);
    let ger: Address = env("BESU_GER_ADDR").parse().unwrap();
    let bridge: Address = env("BESU_BRIDGE_ADDR").parse().unwrap();
    assert_eq!(ger, Address::from(L2_GER_ADDR));
    let out_dir = std::env::var("BESU_OUT_DIR").ok();
    let caller = Address::ZERO;

    let provider = ProviderBuilder::new().connect_http(rpc.clone());

    for n in env("BESU_BLOCKS")
        .split(',')
        .map(|s| s.parse::<u64>().unwrap())
    {
        // Reference values straight from the node.
        let raw: alloy::primitives::Bytes = provider
            .raw_request("debug_getRawHeader".into(), (format!("0x{n:x}"),))
            .await
            .unwrap();
        let anchor_expected: B256 = keccak256(&raw);
        let reported = provider
            .get_block_by_number(n.into())
            .await
            .unwrap()
            .unwrap()
            .header
            .hash;

        let sketch = EvmSketch::builder()
            .eth_like()
            .at_block(n)
            .with_genesis(genesis.clone())
            .el_rpc_url(rpc.clone())
            .build()
            .await
            .expect("builder accepted the Besu header");

        let o = sketch
            .call_raw(&ContractInput::new_call(
                ger,
                caller,
                GerL2::insertedGERHashChainCall {},
            ))
            .await
            .unwrap();
        println!("block {n} ger.insertedGERHashChain = {o}");
        let o = sketch
            .call_raw(&ContractInput::new_call(
                ger,
                caller,
                GerL2::removedGERHashChainCall {},
            ))
            .await
            .unwrap();
        println!("block {n} ger.removedGERHashChain = {o}");
        let o = sketch
            .call_raw(&ContractInput::new_call(
                ger,
                caller,
                GerL2::bridgeAddressCall {},
            ))
            .await
            .unwrap();
        println!("block {n} ger.bridgeAddress = {o}");
        let o = sketch
            .call_raw(&ContractInput::new_call(
                bridge,
                caller,
                BridgeL2SovereignChain::getRootCall {},
            ))
            .await
            .unwrap();
        println!("block {n} bridge.getRoot = {o}");
        let o = sketch
            .call_raw(&ContractInput::new_call(
                bridge,
                caller,
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            ))
            .await
            .unwrap();
        println!("block {n} bridge.claimedGlobalIndexHashChain = {o}");
        let o = sketch
            .call_raw(&ContractInput::new_call(
                bridge,
                caller,
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            ))
            .await
            .unwrap();
        println!("block {n} bridge.unsetGlobalIndexHashChain = {o}");

        let input = sketch.finalize().await.expect("finalize");

        // Client side, the way `StaticCallWithContext` does it.
        let client = ClientExecutor::eth_like(&input).expect("client init");
        let pv = client
            .execute(ContractInput::new_call(
                ger,
                caller,
                GerL2::bridgeAddressCall {},
            ))
            .expect("client execute");
        let anchor_derived = B256::from(pv.anchorHash.0);
        println!("block {n}: anchor derived by sp1-cc = {anchor_derived}");
        println!("block {n}: keccak256(debug_getRawHeader) = {anchor_expected}");
        println!("block {n}: hash reported by eth_getBlockByNumber = {reported}");
        assert_eq!(anchor_derived, anchor_expected);

        let bytes = bincode::sp1_compatible().serialize(&input).unwrap();
        println!("block {n}: sketch bincode bytes = {}", bytes.len());
        if let Some(dir) = &out_dir {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(format!("{dir}/block-{n}.bin"), bytes).unwrap();
        }
    }
}
