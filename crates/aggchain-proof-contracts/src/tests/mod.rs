mod aggchain_contracts_rpc_client {
    use std::str::FromStr;

    use alloy::primitives::{address, B256};
    use mockito::ServerGuard;
    use prover_alloy::AlloyFillProvider;
    use serde_json::json;
    use url::Url;

    use crate::config::AggchainProofContractsConfig;
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

        let mock_server_l1_url = Url::parse(&server_l1.url()).unwrap();
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
        Ok((
            result?,
            TestServers {
                server_l1,
                server_l2_el,
                server_l2_cl,
            },
        ))
    }

    #[test]
    fn parsing_l2_output_root() -> Result<(), Box<dyn std::error::Error>> {
        let json_str = r#"{"version":"0x0000000000000000000000000000000000000000000000000000000000000000",
        "outputRoot":"0xf9758545eb67c1a90276b44bb80047fa72148f88c69a8653f36cd157f537bde4",
        "blockRef":{"hash":"0x2d0d159b47e89cd85b82c18d217fa47f5901e81e71ae80356854849656b43354","number":16, "parentHash":"0xa64a3a659538ce9fa247ddcd83a6ce51442de0047e5a01aa6c11833493819831","timestamp":1740413570,
        "l1origin":{"hash":"0xa3a945dcb7f80f558631917e37e7b633fb430347a303a025b944bdfdc5f4c5a3","number":13},"sequenceNumber":8},
        "withdrawalStorageRoot":"0x8ed4baae3a927be3dea54996b4d5899f8c01e7594bf50b17dc1e741388ce3d12",
        "stateRoot":"0x965938f5738ae8c501d7de25f61597ea2a76ea2802fc535233eb427376b43328",
        "syncStatus":{"current_l1":{"hash":"0x62735836100c2257429e9cfb80250b5d0df29d0ea9c6bcf0f6da8c2a355115d8","number":29,"parentHash":"0xb036ae96aabc83a1b43ced9de73f4e8b8288cdffcbfbcc29a62ae015ec2d696c","timestamp":1740413646},"current_l1_finalized":{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":0},"head_l1":{"hash":"0x907f61e597d733f603350f7a9eae4a2b444ebda317460f3d67e8d2e3eb53466c","number":30,"parentHash":"0x62735836100c2257429e9cfb80250b5d0df29d0ea9c6bcf0f6da8c2a355115d8","timestamp":1740413652},"safe_l1":{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":0},"finalized_l1":{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":0},"unsafe_l2":{"hash":"0x04cc3d231feefa1685bf2ddf192085b72caf80901b2b96416af107347d66c493","number":59,"parentHash":"0x6521911591fd0cada5e864361a84b6bdd6619b4103011c1cab34b07488041d05","timestamp":1740413656,"l1origin":{"hash":"0xe35ebcb9669083f8b1c54be0b4b073ea620b5419d9cbcdbce808b21ba1f1a577","number":27},"sequenceNumber":0},"safe_l2":{"hash":"0x049bbccee5391aee272fe5a90e8243b0af5cef59bbf0fb971cb62314d9149780","number":41,"parentHash":"0xd47dff00fd82e599635eac5139ddea0070d57cd1111be64c604db8cc64f2396b","timestamp":1740413620,"l1origin":{"hash":"0xdb22ab9cb1619cd14f3e9092c7729faf0efa711afcc7f5440b44d5ea6263239e","number":21},"sequenceNumber":0},"finalized_l2":{"hash":"0x0dc6c55aa95cce979b62983736b1ca066560db65bac8efbb4050ddb412b4ad8c","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":1740413538,"l1origin":{"hash":"0x12ec696bd1cb110f89513b620071935f09460ee6a81fea25fc2f400ea816bb7b","number":11},"sequenceNumber":0},"pending_safe_l2":{"hash":"0x049bbccee5391aee272fe5a90e8243b0af5cef59bbf0fb971cb62314d9149780","number":41,"parentHash":"0xd47dff00fd82e599635eac5139ddea0070d57cd1111be64c604db8cc64f2396b","timestamp":1740413620,"l1origin":{"hash":"0xdb22ab9cb1619cd14f3e9092c7729faf0efa711afcc7f5440b44d5ea6263239e","number":21},"sequenceNumber":0},"cross_unsafe_l2":{"hash":"0x04cc3d231feefa1685bf2ddf192085b72caf80901b2b96416af107347d66c493","number":59,"parentHash":"0x6521911591fd0cada5e864361a84b6bdd6619b4103011c1cab34b07488041d05","timestamp":1740413656,"l1origin":{"hash":"0xe35ebcb9669083f8b1c54be0b4b073ea620b5419d9cbcdbce808b21ba1f1a577","number":27},"sequenceNumber":0},"local_safe_l2":{"hash":"0x049bbccee5391aee272fe5a90e8243b0af5cef59bbf0fb971cb62314d9149780","number":41,"parentHash":"0xd47dff00fd82e599635eac5139ddea0070d57cd1111be64c604db8cc64f2396b","timestamp":1740413620,
        "l1origin":{"hash":"0xdb22ab9cb1619cd14f3e9092c7729faf0efa711afcc7f5440b44d5ea6263239e","number":21},"sequenceNumber":0}}}"#;
        let result = AggchainContractsRpcClient::<AlloyFillProvider>::parse_l2_output_root(
            serde_json::from_str(json_str).unwrap(),
        )?;

        assert_eq!(B256::from(result.version.0), B256::default());
        assert_eq!(
            B256::from(result.latest_block_hash.0),
            B256::from_str("0x2d0d159b47e89cd85b82c18d217fa47f5901e81e71ae80356854849656b43354")
                .unwrap()
        );
        assert_eq!(
            B256::from(result.output_root.0),
            B256::from_str("0xf9758545eb67c1a90276b44bb80047fa72148f88c69a8653f36cd157f537bde4")
                .unwrap()
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

        let mock_server_l2_url = Url::parse(&server_l2.url()).unwrap();
        let config = AggchainProofContractsConfig {
            l1_rpc_endpoint: dummy_url(),
            l2_execution_layer_rpc_endpoint: mock_server_l2_url.clone(),
            l2_consensus_layer_rpc_endpoint: dummy_url(),
            polygon_rollup_manager: dummy_address(),
            global_exit_root_manager_v2_sovereign_chain: dummy_address(),
        };

        let result = AggchainContractsRpcClient::new(1, &config).await;

        mock_l2.assert_async().await;
        match result {
            Err(crate::Error::BridgeAddressError(_)) => Ok(()),
            Err(e) => panic!("Expected BridgeAddressError, got {:?}", e),
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
            aggchain_proof_core::Digest(
                B256::from_str(
                    "0x27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"
                )
                .unwrap()
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
                panic!(
                    "Expected alloy transport deserialization error, got {:?}",
                    error
                );
            }
            Err(e) => panic!("Expected LocalExitRootError, got {:?}", e),
            Ok(_) => panic!("Expected LocalExitRootError, got valid Digest"),
        }
    }

    #[test_log::test(tokio::test)]
    async fn get_rollup_config_hash() -> Result<(), Box<dyn std::error::Error>> {
        let (contracts_client, test_servers) = aggchain_contracts_rpc_client().await?;
        let mut server_l1 = test_servers.server_l1;

        // We ask the PolygonZkEVMBridgeV2 for the local exit root with `getRoot()`
        let get_rollup_config_hash = serde_json::json!({
            "method": "eth_call",
            "params": [{
                "to":"0x8e80ffe6dc044f4a766afd6e5a8732fe0977a493",
                "input":"0xb7fd13ce",
            },
            "latest"],
            "id": 1,
            "jsonrpc": "2.0",
        });

        let mock_l1 = server_l1
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(get_rollup_config_hash))
            .with_body(
                json!({
                    "id": 1,
                    "jsonrpc": "2.0",
                    "result": "0xaaaeffa0811291c96c8cbddcc148bf48a6d68c7974b94356f53754ef617122dd"
                })
                .to_string(),
            )
            .create();

        let result = contracts_client.get_rollup_config_hash().await;

        mock_l1.assert_async().await;

        let rollup_config_hash = result?;
        assert_eq!(
            rollup_config_hash,
            aggchain_proof_core::Digest(
                B256::from_str(
                    "0xaaaeffa0811291c96c8cbddcc148bf48a6d68c7974b94356f53754ef617122dd"
                )
                .unwrap()
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
            "method": "l2_outputAtBlock",
            "params":["0x10"],
            "id": 0,
            "jsonrpc": "2.0",
        });

        let mock_l2_cl = server_l2_cl
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(get_rollup_config_hash))
            .with_body(
                json!({
                      "jsonrpc": "2.0",
                      "id": 0,
                      "result": {
                        "version": "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "outputRoot": "0x0151d2aa6a406444c37ebb2a155201863f1626018e1b43a16ca7a160380417b1",
                        "blockRef": {
                          "hash": "0x23b4dee9110bcd5e4befdb5b112a91298ed0058bfb933bdcc8abedce8affa3ff",
                          "number": 16,
                          "parentHash": "0xe6dd9f1b4226427e8921affb7882b710f9d3865f74aa710376981ff0201c7eef",
                          "timestamp": 1741185812,
                          "l1origin": {
                            "hash": "0xe1aafecedaab501145a0273f0f1ff9be1ab60973e87b9c8ab3c54954d6a73f5f",
                            "number": 13
                          },
                          "sequenceNumber": 8
                        },
                        "withdrawalStorageRoot": "0x8ed4baae3a927be3dea54996b4d5899f8c01e7594bf50b17dc1e741388ce3d12",
                        "stateRoot": "0x119ba90b2c0086a7e6520cc9ceacba505e10909820dc115b933f37b08343edcc",
                        "syncStatus": {
                          "current_l1": {
                            "hash": "0x722c70358d6b2ea4c3353d9490907128e3f4eca1f99b9ddfb984f4418fa0a3f4",
                            "number": 149,
                            "parentHash": "0x279854203f996618b021c8c95ce0ec01ee4d6d0a2ecd71a3b21f5000d5c6df84",
                            "timestamp": 1741186608
                          },
                          "current_l1_finalized": {
                            "hash": "0x0c357b1a0af6309d8ab78b3ef2f4a9daf59b4729e7ab04512afb8913086beef8",
                            "number": 124,
                            "parentHash": "0x34fe2a51cf3055c5ecb4bf34d6894c1997b55a9c285b75ac865585b1e0836bd0",
                            "timestamp": 1741186458
                          },
                          "head_l1": {
                            "hash": "0x8429b918847d0ab021a189c63a3afe3ced05f52d4183e49cc3517f89ccab72a4",
                            "number": 150,
                            "parentHash": "0x722c70358d6b2ea4c3353d9490907128e3f4eca1f99b9ddfb984f4418fa0a3f4",
                            "timestamp": 1741186614
                          },
                          "safe_l1": {
                            "hash": "0x503dba01b3660227612864f3d25cca5d8093edc55ad50384e9db4209aad1f092",
                            "number": 132,
                            "parentHash": "0x96d08e94b9de28096bfefb672e1d30d3edd85d065d56a27a66678933bdff354c",
                            "timestamp": 1741186506
                          },
                          "finalized_l1": {
                            "hash": "0x0c357b1a0af6309d8ab78b3ef2f4a9daf59b4729e7ab04512afb8913086beef8",
                            "number": 124,
                            "parentHash": "0x34fe2a51cf3055c5ecb4bf34d6894c1997b55a9c285b75ac865585b1e0836bd0",
                            "timestamp": 1741186458
                          },
                          "unsafe_l2": {
                            "hash": "0x740213cefd9f67e92cd4b11ce080aa3df2588fbea322180907c3be1b556f9037",
                            "number": 421,
                            "parentHash": "0x985e03ddac97a61e8bf4b578924224c533595dd28f93581c442889ebea3f5cdb",
                            "timestamp": 1741186622,
                            "l1origin": {
                              "hash": "0x279854203f996618b021c8c95ce0ec01ee4d6d0a2ecd71a3b21f5000d5c6df84",
                              "number": 148
                            },
                            "sequenceNumber": 1
                          },
                          "safe_l2": {
                            "hash": "0xec96bf1dfd70a487eb3294b1c9671df069ea9649946d0084716b5ca87dde120f",
                            "number": 402,
                            "parentHash": "0xd8b7d0bfbe6ef48c8ac11b6aab41a21b10a44c63677a151cd8673fa25a741261",
                            "timestamp": 1741186584,
                            "l1origin": {
                              "hash": "0x0dfea5633654803ef2a49c22a668702fe513ff9d33ce0a0195a8c32da913ea0d",
                              "number": 142
                            },
                            "sequenceNumber": 0
                          },
                          "finalized_l2": {
                            "hash": "0xcad397496ca47129895282a0d687551c361ec27dadfd6487b099528138aa4a5e",
                            "number": 330,
                            "parentHash": "0xdefc7a6dd302f4d0fe0850578c35cf3586f738094e13229a2dcd029f017650ac",
                            "timestamp": 1741186440,
                            "l1origin": {
                              "hash": "0x25bfbedfdc3807233c2f232095b92079635d0dae16c8c29545656a84b104651f",
                              "number": 118
                            },
                            "sequenceNumber": 0
                          },
                          "pending_safe_l2": {
                            "hash": "0xec96bf1dfd70a487eb3294b1c9671df069ea9649946d0084716b5ca87dde120f",
                            "number": 402,
                            "parentHash": "0xd8b7d0bfbe6ef48c8ac11b6aab41a21b10a44c63677a151cd8673fa25a741261",
                            "timestamp": 1741186584,
                            "l1origin": {
                              "hash": "0x0dfea5633654803ef2a49c22a668702fe513ff9d33ce0a0195a8c32da913ea0d",
                              "number": 142
                            },
                            "sequenceNumber": 0
                          },
                          "cross_unsafe_l2": {
                            "hash": "0x740213cefd9f67e92cd4b11ce080aa3df2588fbea322180907c3be1b556f9037",
                            "number": 421,
                            "parentHash": "0x985e03ddac97a61e8bf4b578924224c533595dd28f93581c442889ebea3f5cdb",
                            "timestamp": 1741186622,
                            "l1origin": {
                              "hash": "0x279854203f996618b021c8c95ce0ec01ee4d6d0a2ecd71a3b21f5000d5c6df84",
                              "number": 148
                            },
                            "sequenceNumber": 1
                          },
                          "local_safe_l2": {
                            "hash": "0xec96bf1dfd70a487eb3294b1c9671df069ea9649946d0084716b5ca87dde120f",
                            "number": 402,
                            "parentHash": "0xd8b7d0bfbe6ef48c8ac11b6aab41a21b10a44c63677a151cd8673fa25a741261",
                            "timestamp": 1741186584,
                            "l1origin": {
                              "hash": "0x0dfea5633654803ef2a49c22a668702fe513ff9d33ce0a0195a8c32da913ea0d",
                              "number": 142
                            },
                            "sequenceNumber": 0
                          }
                        }
                      }
                    }
                )
                    .to_string(),
            )
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
