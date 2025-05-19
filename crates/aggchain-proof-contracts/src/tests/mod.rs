mod aggchain_contracts_rpc_client {
    use std::str::FromStr;

    use agglayer_interop::types::Digest;
    use alloy::hex::{self, FromHex};
    use alloy::primitives::{address, B256};
    use alloy::sol_types::{SolCall, SolValue};
    use mockito::ServerGuard;
    use prover_alloy::{AlloyFillProvider, L1RpcEndpoint};
    use serde_json::json;
    use url::Url;

    use crate::config::AggchainProofContractsConfig;
    use crate::contracts::AggchainFep::trustedSequencerCall;
    use crate::contracts::{
        L1RollupConfigHashFetcher, L2LocalExitRootFetcher, L2OutputAtBlockFetcher,
    };
    use crate::AggchainContractsRpcClient;

    fn dummy_url() -> Url {
        Url::parse("http://0.0.0.0:0").unwrap()
    }

    fn dummy_address() -> alloy::primitives::Address {
        address!("0x0000000000000000000000000000000000000000")
    }

    struct TestServers {
        pub server_l1: ServerGuard,
        pub server_l2_el: ServerGuard,
        pub server_l2_cl: ServerGuard,
    }

    async fn aggchain_contracts_rpc_client(
    ) -> Result<(AggchainContractsRpcClient<AlloyFillProvider>, TestServers), crate::Error> {
        let mut server_l1 = mockito::Server::new_async().await;
        let mut server_l2_el = mockito::Server::new_async().await;
        let server_l2_cl = mockito::Server::new_async().await;

        // We ask the global exit root manager contract for the PolygonZkEVMBridgeV2
        // contract address with the "bridgeAddress()" call
        let bridge_address_expected_body = serde_json::json!({
            "method": "eth_call",
            "params": [{
                "input":"0xa3c573eb",
                "to":"0x610178da211fef7d417bc0e6fed39f05609ad788",
            },
            "latest"],
            "id": 0,
            "jsonrpc": "2.0",
        });

        let mock_l2 = server_l2_el
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(bridge_address_expected_body))
            .with_body(
                json!({
                    "id": 0,
                    "jsonrpc": "2.0",
                    "result": "0x000000000000000000000000d81e7fb88b8e3a6bae6c1b64e11f1a355641fb7c"
                })
                .to_string(),
            )
            .create();

        // We ask the polygon rollup manager contract for the AggchainFep
        // contract address with the "rollupIDToRollupData(network_id)" call
        let aggchain_fep_expected_body = serde_json::json!(
        {
            "method":"eth_call",
            "params":[
                {
                    "to":"0x9a676e781a523b5d0c0e43731313a708cb607508",
                    "input":"0xf9c4c2ae0000000000000000000000000000000000000000000000000000000000000001",
                },
            "latest"],
            "id": 0,
            "jsonrpc": "2.0",
        });
        let mock_l1 = server_l1
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(aggchain_fep_expected_body))
            .with_body(
                json!({
                    "jsonrpc":"2.0",
                    "id": 0,
                    "result":"0x0000000000000000000000008e80ffe6dc044f4a766afd6e5a8732fe0977a49300000000000000000000000000000000000000000000000000000000000003e90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002"
                })
                    .to_string(),
            )
            .create();

        let mock_l1_trusted_sequencer = mock_trusted_sequencer_call(&mut server_l1);

        let mock_server_l1_url = L1RpcEndpoint::from_str(&server_l1.url()).unwrap();
        let mock_server_l2_el_url = Url::parse(&server_l2_el.url()).unwrap();
        let mock_server_l2_cl_url = Url::parse(&server_l2_cl.url()).unwrap();
        let config = AggchainProofContractsConfig {
            l1_rpc_endpoint: mock_server_l1_url,
            l2_execution_layer_rpc_endpoint: mock_server_l2_el_url,
            l2_consensus_layer_rpc_endpoint: mock_server_l2_cl_url,
            polygon_rollup_manager: address!("0x9a676e781a523b5d0c0e43731313a708cb607508"),
            global_exit_root_manager_v2_sovereign_chain: address!(
                "0x610178dA211FEF7D417bC0e6FeD39F05609AD788"
            ),
        };

        let result = AggchainContractsRpcClient::new(1, &config).await;

        mock_l2.assert_async().await;
        mock_l1.assert_async().await;
        mock_l1_trusted_sequencer.assert_async().await;
        Ok((
            result?,
            TestServers {
                server_l1,
                server_l2_el,
                server_l2_cl,
            },
        ))
    }

    fn mock_get_rollup_config_hash(server_l1: &mut ServerGuard) -> mockito::Mock {
        let rollup_config_hash_expected_body = serde_json::json!(
        {
            "method":"eth_call",
            "params":[
                {
                    "to": "0x8e80ffe6dc044f4a766afd6e5a8732fe0977a493",
                    "input": format!("0x{}", hex::encode(crate::contracts::AggchainFep::rollupConfigHashCall{}.abi_encode())),
                },
                "latest"
            ],
            "id": 2,
            "jsonrpc":"2.0",
        });

        let result = json!({
                "jsonrpc":"2.0",
                "id": 2,
                "result": alloy::primitives::FixedBytes::<32>::from_hex("0xaaaeffa0811291c96c8cbddcc148bf48a6d68c7974b94356f53754ef617122dd").unwrap().abi_encode()
            })
            .to_string();

        server_l1
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(rollup_config_hash_expected_body))
            .with_body(result)
            .create()
    }

    fn mock_trusted_sequencer_call(server_l1: &mut ServerGuard) -> mockito::Mock {
        let trusted_sequencer_expected_body = serde_json::json!(
        {
            "method":"eth_call",
            "params":[
                {
                    "to": "0x8e80ffe6dc044f4a766afd6e5a8732fe0977a493",
                    "input": format!("0x{}", hex::encode(trustedSequencerCall{}.abi_encode())),
                },
                "latest"
            ],
            "id": 1,
            "jsonrpc":"2.0",
        });

        let result = json!({
            "jsonrpc":"2.0",
            "id": 1,
            "result": alloy::primitives::Address::from_hex("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap().abi_encode()
        })
        .to_string();

        server_l1
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(trusted_sequencer_expected_body))
            .with_body(result)
            .create()
    }

    #[test]
    fn parsing_l2_output_root() -> Result<(), Box<dyn std::error::Error>> {
        let json_l2_output_root_str = include_str!("parsing_l2_output_root.json");
        let result = AggchainContractsRpcClient::<AlloyFillProvider>::parse_l2_output_root(
            serde_json::from_str(json_l2_output_root_str)?,
        )?;

        assert_eq!(B256::from(result.version.0), B256::default());
        assert_eq!(
            B256::from(result.latest_block_hash.0),
            B256::from_str("0x2d0d159b47e89cd85b82c18d217fa47f5901e81e71ae80356854849656b43354")?
        );
        assert_eq!(
            B256::from(result.output_root.0),
            B256::from_str("0xf9758545eb67c1a90276b44bb80047fa72148f88c69a8653f36cd157f537bde4")?
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_new_contracts_rpc_client() -> Result<(), Box<dyn std::error::Error>> {
        let result = aggchain_contracts_rpc_client().await;

        assert!(result.is_ok());
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_new_contracts_rpc_client_wrong_contract() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut server_l2 = mockito::Server::new_async().await;

        // We ask the global exit root manager contract for the PolygonZkEVMBridgeV2
        // contract address with the "bridgeAddress()" call
        let bridge_address_expected_body = serde_json::json!({
            "method": "eth_call",
            "params": [{
                "input":"0xa3c573eb",
                "to":"0x0000000000000000000000000000000000000000",
            },
            "latest"],
            "id": 0,
            "jsonrpc": "2.0",
        });

        let mock_l2 = server_l2
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(bridge_address_expected_body))
            .with_body(
                json!({
                    "id": 0,
                    "jsonrpc": "2.0",
                    "result": "0x"
                })
                .to_string(),
            )
            .create();

        let mock_server_l2_url = Url::parse(&server_l2.url())?;
        let config = AggchainProofContractsConfig {
            l1_rpc_endpoint: L1RpcEndpoint::from(dummy_url()),
            l2_execution_layer_rpc_endpoint: mock_server_l2_url.clone(),
            l2_consensus_layer_rpc_endpoint: dummy_url(),
            polygon_rollup_manager: dummy_address(),
            global_exit_root_manager_v2_sovereign_chain: dummy_address(),
        };

        let result = AggchainContractsRpcClient::new(1, &config).await;

        mock_l2.assert_async().await;
        match result {
            Err(crate::Error::BridgeAddressError(_)) => Ok(()),
            Err(e) => panic!("Expected BridgeAddressError, got {e:?}"),
            Ok(_) => panic!("Expected BridgeAddressError, got valid client"),
        }
    }

    #[test_log::test(tokio::test)]
    async fn test_get_local_exit_root() -> Result<(), Box<dyn std::error::Error>> {
        let (contracts_client, test_servers) = aggchain_contracts_rpc_client().await?;
        let mut server_l2_el = test_servers.server_l2_el;

        // We ask the PolygonZkEVMBridgeV2 for the local exit root with `getRoot()`
        let get_local_exit_root_body = serde_json::json!({
            "method": "eth_call",
            "params": [{
                "to":"0xd81e7fb88b8e3a6bae6c1b64e11f1a355641fb7c",
                "input":"0x5ca1e165",
            },
            "0xa"],
            "id": 1,
            "jsonrpc": "2.0",
        });

        let mock_l2 = server_l2_el
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(get_local_exit_root_body))
            .with_body(
                json!({
                    "id": 1,
                    "jsonrpc": "2.0",
                    "result": "0x27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"
                })
                .to_string(),
            )
            .create();

        let result = contracts_client.get_l2_local_exit_root(10).await;

        mock_l2.assert_async().await;
        let local_exit_root = result?;
        assert_eq!(
            local_exit_root,
            Digest(
                B256::from_str(
                    "0x27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"
                )?
                .0
            )
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_invalid_local_exit_root() -> Result<(), Box<dyn std::error::Error>> {
        let (contracts_client, test_servers) = aggchain_contracts_rpc_client().await?;
        let mut server_l2_el = test_servers.server_l2_el;

        // We ask the PolygonZkEVMBridgeV2 for the local exit root with `getRoot()`
        let get_local_exit_root_body = serde_json::json!({
            "method": "eth_call",
            "params": [{
                "to":"0xd81e7fb88b8e3a6bae6c1b64e11f1a355641fb7c",
                "input":"0x5ca1e165",
            },
            "0xa"],
            "id": 1,
            "jsonrpc": "2.0",
        });

        let mock_l2 = server_l2_el
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(get_local_exit_root_body))
            .with_body(
                json!({
                    "id": 1,
                    "jsonrpc": "2.0",
                    "result": "0x27ae5ba08d7291c96c8cbddcc"
                })
                .to_string(),
            )
            .create();

        let result = contracts_client.get_l2_local_exit_root(10).await;

        mock_l2.assert_async().await;
        match result {
            Err(crate::Error::LocalExitRootError(alloy::contract::Error::TransportError(_))) => {
                Ok(())
            }
            Err(crate::Error::LocalExitRootError(error)) => {
                panic!("Expected alloy transport deserialization error, got {error:?}");
            }
            Err(e) => panic!("Expected LocalExitRootError, got {e:?}"),
            Ok(_) => panic!("Expected LocalExitRootError, got valid Digest"),
        }
    }

    #[test_log::test(tokio::test)]
    async fn get_rollup_config_hash() -> Result<(), Box<dyn std::error::Error>> {
        let (contracts_client, test_servers) = aggchain_contracts_rpc_client().await?;
        let mut server_l1 = test_servers.server_l1;

        let mock_l1 = mock_get_rollup_config_hash(&mut server_l1);
        let result = contracts_client.get_rollup_config_hash().await;

        mock_l1.assert_async().await;

        let rollup_config_hash = result?;
        assert_eq!(
            rollup_config_hash,
            Digest(
                B256::from_str(
                    "0xaaaeffa0811291c96c8cbddcc148bf48a6d68c7974b94356f53754ef617122dd"
                )?
                .0
            )
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn get_l2_output_at_block() -> Result<(), Box<dyn std::error::Error>> {
        let (contracts_client, test_servers) = aggchain_contracts_rpc_client().await?;
        let mut server_l2_cl = test_servers.server_l2_cl;

        // We ask the PolygonZkEVMBridgeV2 for the local exit root with `getRoot()`
        let get_rollup_config_hash = serde_json::json!({
            "method": "optimism_outputAtBlock",
            "params":["0x10"],
            "id": 0,
            "jsonrpc": "2.0",
        });

        let json_str_response = include_str!("get_l2_output_at_block.json");
        let mock_l2_cl = server_l2_cl
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(get_rollup_config_hash))
            .with_body(json_str_response)
            .create();

        let result = contracts_client.get_l2_output_at_block(0x10).await;

        mock_l2_cl.assert_async().await;
        assert!(result.is_ok());
        let output = result?;
        assert_eq!(
            output.output_root.to_string(),
            "0x0151d2aa6a406444c37ebb2a155201863f1626018e1b43a16ca7a160380417b1"
        );
        assert_eq!(
            output.withdrawal_storage_root.to_string(),
            "0x8ed4baae3a927be3dea54996b4d5899f8c01e7594bf50b17dc1e741388ce3d12"
        );
        assert_eq!(
            output.latest_block_hash.to_string(),
            "0x23b4dee9110bcd5e4befdb5b112a91298ed0058bfb933bdcc8abedce8affa3ff"
        );
        assert_eq!(
            output.state_root.to_string(),
            "0x119ba90b2c0086a7e6520cc9ceacba505e10909820dc115b933f37b08343edcc"
        );
        assert_eq!(
            output.version.to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        Ok(())
    }
}
