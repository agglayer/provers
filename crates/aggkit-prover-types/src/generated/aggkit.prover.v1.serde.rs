// @generated
impl serde::Serialize for BridgeExit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.leaf_type != 0 {
            len += 1;
        }
        if self.token_info.is_some() {
            len += 1;
        }
        if self.destination_network != 0 {
            len += 1;
        }
        if !self.destination_address.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        if self.is_metadata_hashed {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.BridgeExit", len)?;
        if self.leaf_type != 0 {
            let v = LeafType::try_from(self.leaf_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.leaf_type)))?;
            struct_ser.serialize_field("leafType", &v)?;
        }
        if let Some(v) = self.token_info.as_ref() {
            struct_ser.serialize_field("tokenInfo", v)?;
        }
        if self.destination_network != 0 {
            struct_ser.serialize_field("destinationNetwork", &self.destination_network)?;
        }
        if !self.destination_address.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("destinationAddress", pbjson::private::base64::encode(&self.destination_address).as_str())?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        if self.is_metadata_hashed {
            struct_ser.serialize_field("isMetadataHashed", &self.is_metadata_hashed)?;
        }
        if !self.metadata.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("metadata", pbjson::private::base64::encode(&self.metadata).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BridgeExit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "leaf_type",
            "leafType",
            "token_info",
            "tokenInfo",
            "destination_network",
            "destinationNetwork",
            "destination_address",
            "destinationAddress",
            "amount",
            "is_metadata_hashed",
            "isMetadataHashed",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LeafType,
            TokenInfo,
            DestinationNetwork,
            DestinationAddress,
            Amount,
            IsMetadataHashed,
            Metadata,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "leafType" | "leaf_type" => Ok(GeneratedField::LeafType),
                            "tokenInfo" | "token_info" => Ok(GeneratedField::TokenInfo),
                            "destinationNetwork" | "destination_network" => Ok(GeneratedField::DestinationNetwork),
                            "destinationAddress" | "destination_address" => Ok(GeneratedField::DestinationAddress),
                            "amount" => Ok(GeneratedField::Amount),
                            "isMetadataHashed" | "is_metadata_hashed" => Ok(GeneratedField::IsMetadataHashed),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BridgeExit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.BridgeExit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BridgeExit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut leaf_type__ = None;
                let mut token_info__ = None;
                let mut destination_network__ = None;
                let mut destination_address__ = None;
                let mut amount__ = None;
                let mut is_metadata_hashed__ = None;
                let mut metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LeafType => {
                            if leaf_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("leafType"));
                            }
                            leaf_type__ = Some(map_.next_value::<LeafType>()? as i32);
                        }
                        GeneratedField::TokenInfo => {
                            if token_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tokenInfo"));
                            }
                            token_info__ = map_.next_value()?;
                        }
                        GeneratedField::DestinationNetwork => {
                            if destination_network__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationNetwork"));
                            }
                            destination_network__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DestinationAddress => {
                            if destination_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationAddress"));
                            }
                            destination_address__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsMetadataHashed => {
                            if is_metadata_hashed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isMetadataHashed"));
                            }
                            is_metadata_hashed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(BridgeExit {
                    leaf_type: leaf_type__.unwrap_or_default(),
                    token_info: token_info__,
                    destination_network: destination_network__.unwrap_or_default(),
                    destination_address: destination_address__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                    is_metadata_hashed: is_metadata_hashed__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.BridgeExit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ClaimFromMainnet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.inclusion_proof.is_some() {
            len += 1;
        }
        if self.l1_leaf.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.ClaimFromMainnet", len)?;
        if let Some(v) = self.inclusion_proof.as_ref() {
            struct_ser.serialize_field("inclusionProof", v)?;
        }
        if let Some(v) = self.l1_leaf.as_ref() {
            struct_ser.serialize_field("l1Leaf", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClaimFromMainnet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inclusion_proof",
            "inclusionProof",
            "l1_leaf",
            "l1Leaf",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InclusionProof,
            L1Leaf,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "inclusionProof" | "inclusion_proof" => Ok(GeneratedField::InclusionProof),
                            "l1Leaf" | "l1_leaf" => Ok(GeneratedField::L1Leaf),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClaimFromMainnet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.ClaimFromMainnet")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ClaimFromMainnet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inclusion_proof__ = None;
                let mut l1_leaf__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InclusionProof => {
                            if inclusion_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inclusionProof"));
                            }
                            inclusion_proof__ = map_.next_value()?;
                        }
                        GeneratedField::L1Leaf => {
                            if l1_leaf__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1Leaf"));
                            }
                            l1_leaf__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ClaimFromMainnet {
                    inclusion_proof: inclusion_proof__,
                    l1_leaf: l1_leaf__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.ClaimFromMainnet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateAggchainProofRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_block != 0 {
            len += 1;
        }
        if self.max_end_block != 0 {
            len += 1;
        }
        if self.l1_info_tree_root_hash.is_some() {
            len += 1;
        }
        if self.l1_info_tree_leaf.is_some() {
            len += 1;
        }
        if !self.l1_info_tree_merkle_proof.is_empty() {
            len += 1;
        }
        if !self.ger_leaves.is_empty() {
            len += 1;
        }
        if !self.imported_bridge_exits.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.GenerateAggchainProofRequest", len)?;
        if self.start_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startBlock", ToString::to_string(&self.start_block).as_str())?;
        }
        if self.max_end_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("maxEndBlock", ToString::to_string(&self.max_end_block).as_str())?;
        }
        if let Some(v) = self.l1_info_tree_root_hash.as_ref() {
            struct_ser.serialize_field("l1InfoTreeRootHash", v)?;
        }
        if let Some(v) = self.l1_info_tree_leaf.as_ref() {
            struct_ser.serialize_field("l1InfoTreeLeaf", v)?;
        }
        if !self.l1_info_tree_merkle_proof.is_empty() {
            struct_ser.serialize_field("l1InfoTreeMerkleProof", &self.l1_info_tree_merkle_proof.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        if !self.ger_leaves.is_empty() {
            struct_ser.serialize_field("gerLeaves", &self.ger_leaves)?;
        }
        if !self.imported_bridge_exits.is_empty() {
            struct_ser.serialize_field("importedBridgeExits", &self.imported_bridge_exits)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateAggchainProofRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_block",
            "startBlock",
            "max_end_block",
            "maxEndBlock",
            "l1_info_tree_root_hash",
            "l1InfoTreeRootHash",
            "l1_info_tree_leaf",
            "l1InfoTreeLeaf",
            "l1_info_tree_merkle_proof",
            "l1InfoTreeMerkleProof",
            "ger_leaves",
            "gerLeaves",
            "imported_bridge_exits",
            "importedBridgeExits",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartBlock,
            MaxEndBlock,
            L1InfoTreeRootHash,
            L1InfoTreeLeaf,
            L1InfoTreeMerkleProof,
            GerLeaves,
            ImportedBridgeExits,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "startBlock" | "start_block" => Ok(GeneratedField::StartBlock),
                            "maxEndBlock" | "max_end_block" => Ok(GeneratedField::MaxEndBlock),
                            "l1InfoTreeRootHash" | "l1_info_tree_root_hash" => Ok(GeneratedField::L1InfoTreeRootHash),
                            "l1InfoTreeLeaf" | "l1_info_tree_leaf" => Ok(GeneratedField::L1InfoTreeLeaf),
                            "l1InfoTreeMerkleProof" | "l1_info_tree_merkle_proof" => Ok(GeneratedField::L1InfoTreeMerkleProof),
                            "gerLeaves" | "ger_leaves" => Ok(GeneratedField::GerLeaves),
                            "importedBridgeExits" | "imported_bridge_exits" => Ok(GeneratedField::ImportedBridgeExits),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateAggchainProofRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.GenerateAggchainProofRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateAggchainProofRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_block__ = None;
                let mut max_end_block__ = None;
                let mut l1_info_tree_root_hash__ = None;
                let mut l1_info_tree_leaf__ = None;
                let mut l1_info_tree_merkle_proof__ = None;
                let mut ger_leaves__ = None;
                let mut imported_bridge_exits__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StartBlock => {
                            if start_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startBlock"));
                            }
                            start_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxEndBlock => {
                            if max_end_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxEndBlock"));
                            }
                            max_end_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::L1InfoTreeRootHash => {
                            if l1_info_tree_root_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1InfoTreeRootHash"));
                            }
                            l1_info_tree_root_hash__ = map_.next_value()?;
                        }
                        GeneratedField::L1InfoTreeLeaf => {
                            if l1_info_tree_leaf__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1InfoTreeLeaf"));
                            }
                            l1_info_tree_leaf__ = map_.next_value()?;
                        }
                        GeneratedField::L1InfoTreeMerkleProof => {
                            if l1_info_tree_merkle_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1InfoTreeMerkleProof"));
                            }
                            l1_info_tree_merkle_proof__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::GerLeaves => {
                            if ger_leaves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gerLeaves"));
                            }
                            ger_leaves__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::ImportedBridgeExits => {
                            if imported_bridge_exits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importedBridgeExits"));
                            }
                            imported_bridge_exits__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GenerateAggchainProofRequest {
                    start_block: start_block__.unwrap_or_default(),
                    max_end_block: max_end_block__.unwrap_or_default(),
                    l1_info_tree_root_hash: l1_info_tree_root_hash__,
                    l1_info_tree_leaf: l1_info_tree_leaf__,
                    l1_info_tree_merkle_proof: l1_info_tree_merkle_proof__.unwrap_or_default(),
                    ger_leaves: ger_leaves__.unwrap_or_default(),
                    imported_bridge_exits: imported_bridge_exits__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.GenerateAggchainProofRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateAggchainProofResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.aggchain_proof.is_empty() {
            len += 1;
        }
        if self.start_block != 0 {
            len += 1;
        }
        if self.end_block != 0 {
            len += 1;
        }
        if self.local_exit_root_hash.is_some() {
            len += 1;
        }
        if !self.custom_chain_data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.GenerateAggchainProofResponse", len)?;
        if !self.aggchain_proof.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("aggchainProof", pbjson::private::base64::encode(&self.aggchain_proof).as_str())?;
        }
        if self.start_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startBlock", ToString::to_string(&self.start_block).as_str())?;
        }
        if self.end_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("endBlock", ToString::to_string(&self.end_block).as_str())?;
        }
        if let Some(v) = self.local_exit_root_hash.as_ref() {
            struct_ser.serialize_field("localExitRootHash", v)?;
        }
        if !self.custom_chain_data.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("customChainData", pbjson::private::base64::encode(&self.custom_chain_data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateAggchainProofResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "aggchain_proof",
            "aggchainProof",
            "start_block",
            "startBlock",
            "end_block",
            "endBlock",
            "local_exit_root_hash",
            "localExitRootHash",
            "custom_chain_data",
            "customChainData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AggchainProof,
            StartBlock,
            EndBlock,
            LocalExitRootHash,
            CustomChainData,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "aggchainProof" | "aggchain_proof" => Ok(GeneratedField::AggchainProof),
                            "startBlock" | "start_block" => Ok(GeneratedField::StartBlock),
                            "endBlock" | "end_block" => Ok(GeneratedField::EndBlock),
                            "localExitRootHash" | "local_exit_root_hash" => Ok(GeneratedField::LocalExitRootHash),
                            "customChainData" | "custom_chain_data" => Ok(GeneratedField::CustomChainData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateAggchainProofResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.GenerateAggchainProofResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateAggchainProofResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut aggchain_proof__ = None;
                let mut start_block__ = None;
                let mut end_block__ = None;
                let mut local_exit_root_hash__ = None;
                let mut custom_chain_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AggchainProof => {
                            if aggchain_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggchainProof"));
                            }
                            aggchain_proof__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StartBlock => {
                            if start_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startBlock"));
                            }
                            start_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndBlock => {
                            if end_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endBlock"));
                            }
                            end_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LocalExitRootHash => {
                            if local_exit_root_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("localExitRootHash"));
                            }
                            local_exit_root_hash__ = map_.next_value()?;
                        }
                        GeneratedField::CustomChainData => {
                            if custom_chain_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("customChainData"));
                            }
                            custom_chain_data__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GenerateAggchainProofResponse {
                    aggchain_proof: aggchain_proof__.unwrap_or_default(),
                    start_block: start_block__.unwrap_or_default(),
                    end_block: end_block__.unwrap_or_default(),
                    local_exit_root_hash: local_exit_root_hash__,
                    custom_chain_data: custom_chain_data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.GenerateAggchainProofResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GerLeaf {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.l1_info_tree_leaf.is_some() {
            len += 1;
        }
        if self.inclusion_proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.GerLeaf", len)?;
        if let Some(v) = self.l1_info_tree_leaf.as_ref() {
            struct_ser.serialize_field("l1InfoTreeLeaf", v)?;
        }
        if let Some(v) = self.inclusion_proof.as_ref() {
            struct_ser.serialize_field("inclusionProof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GerLeaf {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "l1_info_tree_leaf",
            "l1InfoTreeLeaf",
            "inclusion_proof",
            "inclusionProof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            L1InfoTreeLeaf,
            InclusionProof,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "l1InfoTreeLeaf" | "l1_info_tree_leaf" => Ok(GeneratedField::L1InfoTreeLeaf),
                            "inclusionProof" | "inclusion_proof" => Ok(GeneratedField::InclusionProof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GerLeaf;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.GerLeaf")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GerLeaf, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut l1_info_tree_leaf__ = None;
                let mut inclusion_proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::L1InfoTreeLeaf => {
                            if l1_info_tree_leaf__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1InfoTreeLeaf"));
                            }
                            l1_info_tree_leaf__ = map_.next_value()?;
                        }
                        GeneratedField::InclusionProof => {
                            if inclusion_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inclusionProof"));
                            }
                            inclusion_proof__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GerLeaf {
                    l1_info_tree_leaf: l1_info_tree_leaf__,
                    inclusion_proof: inclusion_proof__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.GerLeaf", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GlobalIndex {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.mainnet_flag {
            len += 1;
        }
        if self.rollup_index != 0 {
            len += 1;
        }
        if self.leaf_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.GlobalIndex", len)?;
        if self.mainnet_flag {
            struct_ser.serialize_field("mainnetFlag", &self.mainnet_flag)?;
        }
        if self.rollup_index != 0 {
            struct_ser.serialize_field("rollupIndex", &self.rollup_index)?;
        }
        if self.leaf_index != 0 {
            struct_ser.serialize_field("leafIndex", &self.leaf_index)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GlobalIndex {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "mainnet_flag",
            "mainnetFlag",
            "rollup_index",
            "rollupIndex",
            "leaf_index",
            "leafIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MainnetFlag,
            RollupIndex,
            LeafIndex,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "mainnetFlag" | "mainnet_flag" => Ok(GeneratedField::MainnetFlag),
                            "rollupIndex" | "rollup_index" => Ok(GeneratedField::RollupIndex),
                            "leafIndex" | "leaf_index" => Ok(GeneratedField::LeafIndex),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GlobalIndex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.GlobalIndex")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GlobalIndex, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mainnet_flag__ = None;
                let mut rollup_index__ = None;
                let mut leaf_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MainnetFlag => {
                            if mainnet_flag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mainnetFlag"));
                            }
                            mainnet_flag__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RollupIndex => {
                            if rollup_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rollupIndex"));
                            }
                            rollup_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LeafIndex => {
                            if leaf_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("leafIndex"));
                            }
                            leaf_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GlobalIndex {
                    mainnet_flag: mainnet_flag__.unwrap_or_default(),
                    rollup_index: rollup_index__.unwrap_or_default(),
                    leaf_index: leaf_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.GlobalIndex", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ImportedBridgeExit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.bridge_exit.is_some() {
            len += 1;
        }
        if self.global_index.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.ImportedBridgeExit", len)?;
        if let Some(v) = self.bridge_exit.as_ref() {
            struct_ser.serialize_field("bridgeExit", v)?;
        }
        if let Some(v) = self.global_index.as_ref() {
            struct_ser.serialize_field("globalIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ImportedBridgeExit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "bridge_exit",
            "bridgeExit",
            "global_index",
            "globalIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BridgeExit,
            GlobalIndex,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "bridgeExit" | "bridge_exit" => Ok(GeneratedField::BridgeExit),
                            "globalIndex" | "global_index" => Ok(GeneratedField::GlobalIndex),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ImportedBridgeExit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.ImportedBridgeExit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ImportedBridgeExit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut bridge_exit__ = None;
                let mut global_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BridgeExit => {
                            if bridge_exit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bridgeExit"));
                            }
                            bridge_exit__ = map_.next_value()?;
                        }
                        GeneratedField::GlobalIndex => {
                            if global_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("globalIndex"));
                            }
                            global_index__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ImportedBridgeExit {
                    bridge_exit: bridge_exit__,
                    global_index: global_index__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.ImportedBridgeExit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InclusionProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.siblings.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.InclusionProof", len)?;
        if !self.siblings.is_empty() {
            struct_ser.serialize_field("siblings", &self.siblings.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InclusionProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "siblings",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Siblings,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "siblings" => Ok(GeneratedField::Siblings),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InclusionProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.InclusionProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InclusionProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut siblings__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Siblings => {
                            if siblings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("siblings"));
                            }
                            siblings__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(InclusionProof {
                    siblings: siblings__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.InclusionProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for L1InfoTreeLeaf {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.l1_info_tree_index != 0 {
            len += 1;
        }
        if !self.rer.is_empty() {
            len += 1;
        }
        if !self.mer.is_empty() {
            len += 1;
        }
        if self.inner.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.L1InfoTreeLeaf", len)?;
        if self.l1_info_tree_index != 0 {
            struct_ser.serialize_field("l1InfoTreeIndex", &self.l1_info_tree_index)?;
        }
        if !self.rer.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rer", pbjson::private::base64::encode(&self.rer).as_str())?;
        }
        if !self.mer.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("mer", pbjson::private::base64::encode(&self.mer).as_str())?;
        }
        if let Some(v) = self.inner.as_ref() {
            struct_ser.serialize_field("inner", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for L1InfoTreeLeaf {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "l1_info_tree_index",
            "l1InfoTreeIndex",
            "rer",
            "mer",
            "inner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            L1InfoTreeIndex,
            Rer,
            Mer,
            Inner,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "l1InfoTreeIndex" | "l1_info_tree_index" => Ok(GeneratedField::L1InfoTreeIndex),
                            "rer" => Ok(GeneratedField::Rer),
                            "mer" => Ok(GeneratedField::Mer),
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = L1InfoTreeLeaf;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.L1InfoTreeLeaf")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<L1InfoTreeLeaf, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut l1_info_tree_index__ = None;
                let mut rer__ = None;
                let mut mer__ = None;
                let mut inner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::L1InfoTreeIndex => {
                            if l1_info_tree_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1InfoTreeIndex"));
                            }
                            l1_info_tree_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Rer => {
                            if rer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rer"));
                            }
                            rer__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Mer => {
                            if mer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mer"));
                            }
                            mer__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = map_.next_value()?;
                        }
                    }
                }
                Ok(L1InfoTreeLeaf {
                    l1_info_tree_index: l1_info_tree_index__.unwrap_or_default(),
                    rer: rer__.unwrap_or_default(),
                    mer: mer__.unwrap_or_default(),
                    inner: inner__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.L1InfoTreeLeaf", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for L1InfoTreeLeafInner {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.global_exit_root.is_empty() {
            len += 1;
        }
        if !self.block_hash.is_empty() {
            len += 1;
        }
        if self.timestamp != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.L1InfoTreeLeafInner", len)?;
        if !self.global_exit_root.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("globalExitRoot", pbjson::private::base64::encode(&self.global_exit_root).as_str())?;
        }
        if !self.block_hash.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockHash", pbjson::private::base64::encode(&self.block_hash).as_str())?;
        }
        if self.timestamp != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timestamp", ToString::to_string(&self.timestamp).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for L1InfoTreeLeafInner {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "global_exit_root",
            "globalExitRoot",
            "block_hash",
            "blockHash",
            "timestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GlobalExitRoot,
            BlockHash,
            Timestamp,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "globalExitRoot" | "global_exit_root" => Ok(GeneratedField::GlobalExitRoot),
                            "blockHash" | "block_hash" => Ok(GeneratedField::BlockHash),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = L1InfoTreeLeafInner;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.L1InfoTreeLeafInner")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<L1InfoTreeLeafInner, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut global_exit_root__ = None;
                let mut block_hash__ = None;
                let mut timestamp__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GlobalExitRoot => {
                            if global_exit_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("globalExitRoot"));
                            }
                            global_exit_root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BlockHash => {
                            if block_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHash"));
                            }
                            block_hash__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(L1InfoTreeLeafInner {
                    global_exit_root: global_exit_root__.unwrap_or_default(),
                    block_hash: block_hash__.unwrap_or_default(),
                    timestamp: timestamp__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.L1InfoTreeLeafInner", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LeafType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "LEAF_TYPE_UNSPECIFIED",
            Self::Transfer => "LEAF_TYPE_TRANSFER",
            Self::Message => "LEAF_TYPE_MESSAGE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for LeafType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LEAF_TYPE_UNSPECIFIED",
            "LEAF_TYPE_TRANSFER",
            "LEAF_TYPE_MESSAGE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LeafType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "LEAF_TYPE_UNSPECIFIED" => Ok(LeafType::Unspecified),
                    "LEAF_TYPE_TRANSFER" => Ok(LeafType::Transfer),
                    "LEAF_TYPE_MESSAGE" => Ok(LeafType::Message),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for TokenInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.origin_network != 0 {
            len += 1;
        }
        if !self.origin_token_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.TokenInfo", len)?;
        if self.origin_network != 0 {
            struct_ser.serialize_field("originNetwork", &self.origin_network)?;
        }
        if !self.origin_token_address.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("originTokenAddress", pbjson::private::base64::encode(&self.origin_token_address).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TokenInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "origin_network",
            "originNetwork",
            "origin_token_address",
            "originTokenAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OriginNetwork,
            OriginTokenAddress,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "originNetwork" | "origin_network" => Ok(GeneratedField::OriginNetwork),
                            "originTokenAddress" | "origin_token_address" => Ok(GeneratedField::OriginTokenAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TokenInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.TokenInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TokenInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut origin_network__ = None;
                let mut origin_token_address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OriginNetwork => {
                            if origin_network__.is_some() {
                                return Err(serde::de::Error::duplicate_field("originNetwork"));
                            }
                            origin_network__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::OriginTokenAddress => {
                            if origin_token_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("originTokenAddress"));
                            }
                            origin_token_address__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TokenInfo {
                    origin_network: origin_network__.unwrap_or_default(),
                    origin_token_address: origin_token_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.TokenInfo", FIELDS, GeneratedVisitor)
    }
}
