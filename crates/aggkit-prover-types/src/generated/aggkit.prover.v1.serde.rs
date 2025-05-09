// @generated
impl serde::Serialize for GenerateAggchainProofRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.last_proven_block != 0 {
            len += 1;
        }
        if self.requested_end_block != 0 {
            len += 1;
        }
        if self.l1_info_tree_root_hash.is_some() {
            len += 1;
        }
        if self.l1_info_tree_leaf.is_some() {
            len += 1;
        }
        if self.l1_info_tree_merkle_proof.is_some() {
            len += 1;
        }
        if !self.ger_leaves.is_empty() {
            len += 1;
        }
        if !self.imported_bridge_exits.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.GenerateAggchainProofRequest", len)?;
        if self.last_proven_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("lastProvenBlock", ToString::to_string(&self.last_proven_block).as_str())?;
        }
        if self.requested_end_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("requestedEndBlock", ToString::to_string(&self.requested_end_block).as_str())?;
        }
        if let Some(v) = self.l1_info_tree_root_hash.as_ref() {
            struct_ser.serialize_field("l1InfoTreeRootHash", v)?;
        }
        if let Some(v) = self.l1_info_tree_leaf.as_ref() {
            struct_ser.serialize_field("l1InfoTreeLeaf", v)?;
        }
        if let Some(v) = self.l1_info_tree_merkle_proof.as_ref() {
            struct_ser.serialize_field("l1InfoTreeMerkleProof", v)?;
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
            "last_proven_block",
            "lastProvenBlock",
            "requested_end_block",
            "requestedEndBlock",
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
            LastProvenBlock,
            RequestedEndBlock,
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
                            "lastProvenBlock" | "last_proven_block" => Ok(GeneratedField::LastProvenBlock),
                            "requestedEndBlock" | "requested_end_block" => Ok(GeneratedField::RequestedEndBlock),
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
                let mut last_proven_block__ = None;
                let mut requested_end_block__ = None;
                let mut l1_info_tree_root_hash__ = None;
                let mut l1_info_tree_leaf__ = None;
                let mut l1_info_tree_merkle_proof__ = None;
                let mut ger_leaves__ = None;
                let mut imported_bridge_exits__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LastProvenBlock => {
                            if last_proven_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastProvenBlock"));
                            }
                            last_proven_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RequestedEndBlock => {
                            if requested_end_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestedEndBlock"));
                            }
                            requested_end_block__ = 
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
                            l1_info_tree_merkle_proof__ = map_.next_value()?;
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
                    last_proven_block: last_proven_block__.unwrap_or_default(),
                    requested_end_block: requested_end_block__.unwrap_or_default(),
                    l1_info_tree_root_hash: l1_info_tree_root_hash__,
                    l1_info_tree_leaf: l1_info_tree_leaf__,
                    l1_info_tree_merkle_proof: l1_info_tree_merkle_proof__,
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
        if self.aggchain_proof.is_some() {
            len += 1;
        }
        if self.last_proven_block != 0 {
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
        if let Some(v) = self.aggchain_proof.as_ref() {
            struct_ser.serialize_field("aggchainProof", v)?;
        }
        if self.last_proven_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("lastProvenBlock", ToString::to_string(&self.last_proven_block).as_str())?;
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
            "last_proven_block",
            "lastProvenBlock",
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
            LastProvenBlock,
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
                            "lastProvenBlock" | "last_proven_block" => Ok(GeneratedField::LastProvenBlock),
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
                let mut last_proven_block__ = None;
                let mut end_block__ = None;
                let mut local_exit_root_hash__ = None;
                let mut custom_chain_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AggchainProof => {
                            if aggchain_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggchainProof"));
                            }
                            aggchain_proof__ = map_.next_value()?;
                        }
                        GeneratedField::LastProvenBlock => {
                            if last_proven_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastProvenBlock"));
                            }
                            last_proven_block__ = 
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
                    aggchain_proof: aggchain_proof__,
                    last_proven_block: last_proven_block__.unwrap_or_default(),
                    end_block: end_block__.unwrap_or_default(),
                    local_exit_root_hash: local_exit_root_hash__,
                    custom_chain_data: custom_chain_data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.GenerateAggchainProofResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateOptimisticAggchainProofRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.aggchain_proof_request.is_some() {
            len += 1;
        }
        if self.optimistic_mode_signature.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.GenerateOptimisticAggchainProofRequest", len)?;
        if let Some(v) = self.aggchain_proof_request.as_ref() {
            struct_ser.serialize_field("aggchainProofRequest", v)?;
        }
        if let Some(v) = self.optimistic_mode_signature.as_ref() {
            struct_ser.serialize_field("optimisticModeSignature", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateOptimisticAggchainProofRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "aggchain_proof_request",
            "aggchainProofRequest",
            "optimistic_mode_signature",
            "optimisticModeSignature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AggchainProofRequest,
            OptimisticModeSignature,
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
                            "aggchainProofRequest" | "aggchain_proof_request" => Ok(GeneratedField::AggchainProofRequest),
                            "optimisticModeSignature" | "optimistic_mode_signature" => Ok(GeneratedField::OptimisticModeSignature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateOptimisticAggchainProofRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.GenerateOptimisticAggchainProofRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateOptimisticAggchainProofRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut aggchain_proof_request__ = None;
                let mut optimistic_mode_signature__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AggchainProofRequest => {
                            if aggchain_proof_request__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggchainProofRequest"));
                            }
                            aggchain_proof_request__ = map_.next_value()?;
                        }
                        GeneratedField::OptimisticModeSignature => {
                            if optimistic_mode_signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("optimisticModeSignature"));
                            }
                            optimistic_mode_signature__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GenerateOptimisticAggchainProofRequest {
                    aggchain_proof_request: aggchain_proof_request__,
                    optimistic_mode_signature: optimistic_mode_signature__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.GenerateOptimisticAggchainProofRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateOptimisticAggchainProofResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.aggchain_proof.is_some() {
            len += 1;
        }
        if self.local_exit_root_hash.is_some() {
            len += 1;
        }
        if !self.custom_chain_data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.GenerateOptimisticAggchainProofResponse", len)?;
        if let Some(v) = self.aggchain_proof.as_ref() {
            struct_ser.serialize_field("aggchainProof", v)?;
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
impl<'de> serde::Deserialize<'de> for GenerateOptimisticAggchainProofResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "aggchain_proof",
            "aggchainProof",
            "local_exit_root_hash",
            "localExitRootHash",
            "custom_chain_data",
            "customChainData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AggchainProof,
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
            type Value = GenerateOptimisticAggchainProofResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.GenerateOptimisticAggchainProofResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateOptimisticAggchainProofResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut aggchain_proof__ = None;
                let mut local_exit_root_hash__ = None;
                let mut custom_chain_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AggchainProof => {
                            if aggchain_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggchainProof"));
                            }
                            aggchain_proof__ = map_.next_value()?;
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
                Ok(GenerateOptimisticAggchainProofResponse {
                    aggchain_proof: aggchain_proof__,
                    local_exit_root_hash: local_exit_root_hash__,
                    custom_chain_data: custom_chain_data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.GenerateOptimisticAggchainProofResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ImportedBridgeExitWithBlockNumber {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_number != 0 {
            len += 1;
        }
        if self.global_index.is_some() {
            len += 1;
        }
        if self.bridge_exit_hash.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.ImportedBridgeExitWithBlockNumber", len)?;
        if self.block_number != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockNumber", ToString::to_string(&self.block_number).as_str())?;
        }
        if let Some(v) = self.global_index.as_ref() {
            struct_ser.serialize_field("globalIndex", v)?;
        }
        if let Some(v) = self.bridge_exit_hash.as_ref() {
            struct_ser.serialize_field("bridgeExitHash", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ImportedBridgeExitWithBlockNumber {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_number",
            "blockNumber",
            "global_index",
            "globalIndex",
            "bridge_exit_hash",
            "bridgeExitHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockNumber,
            GlobalIndex,
            BridgeExitHash,
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
                            "blockNumber" | "block_number" => Ok(GeneratedField::BlockNumber),
                            "globalIndex" | "global_index" => Ok(GeneratedField::GlobalIndex),
                            "bridgeExitHash" | "bridge_exit_hash" => Ok(GeneratedField::BridgeExitHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ImportedBridgeExitWithBlockNumber;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.ImportedBridgeExitWithBlockNumber")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ImportedBridgeExitWithBlockNumber, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_number__ = None;
                let mut global_index__ = None;
                let mut bridge_exit_hash__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BlockNumber => {
                            if block_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockNumber"));
                            }
                            block_number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GlobalIndex => {
                            if global_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("globalIndex"));
                            }
                            global_index__ = map_.next_value()?;
                        }
                        GeneratedField::BridgeExitHash => {
                            if bridge_exit_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bridgeExitHash"));
                            }
                            bridge_exit_hash__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ImportedBridgeExitWithBlockNumber {
                    block_number: block_number__.unwrap_or_default(),
                    global_index: global_index__,
                    bridge_exit_hash: bridge_exit_hash__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.ImportedBridgeExitWithBlockNumber", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProvenInsertedGer {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proof_ger_l1root.is_some() {
            len += 1;
        }
        if self.l1_leaf.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.ProvenInsertedGER", len)?;
        if let Some(v) = self.proof_ger_l1root.as_ref() {
            struct_ser.serialize_field("proofGerL1root", v)?;
        }
        if let Some(v) = self.l1_leaf.as_ref() {
            struct_ser.serialize_field("l1Leaf", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProvenInsertedGer {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proof_ger_l1root",
            "proofGerL1root",
            "l1_leaf",
            "l1Leaf",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProofGerL1root,
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
                            "proofGerL1root" | "proof_ger_l1root" => Ok(GeneratedField::ProofGerL1root),
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
            type Value = ProvenInsertedGer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.ProvenInsertedGER")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProvenInsertedGer, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof_ger_l1root__ = None;
                let mut l1_leaf__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ProofGerL1root => {
                            if proof_ger_l1root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofGerL1root"));
                            }
                            proof_ger_l1root__ = map_.next_value()?;
                        }
                        GeneratedField::L1Leaf => {
                            if l1_leaf__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1Leaf"));
                            }
                            l1_leaf__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ProvenInsertedGer {
                    proof_ger_l1root: proof_ger_l1root__,
                    l1_leaf: l1_leaf__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.ProvenInsertedGER", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProvenInsertedGerWithBlockNumber {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_number != 0 {
            len += 1;
        }
        if self.proven_inserted_ger.is_some() {
            len += 1;
        }
        if self.block_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.ProvenInsertedGERWithBlockNumber", len)?;
        if self.block_number != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockNumber", ToString::to_string(&self.block_number).as_str())?;
        }
        if let Some(v) = self.proven_inserted_ger.as_ref() {
            struct_ser.serialize_field("provenInsertedGer", v)?;
        }
        if self.block_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockIndex", ToString::to_string(&self.block_index).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProvenInsertedGerWithBlockNumber {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_number",
            "blockNumber",
            "proven_inserted_ger",
            "provenInsertedGer",
            "block_index",
            "blockIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockNumber,
            ProvenInsertedGer,
            BlockIndex,
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
                            "blockNumber" | "block_number" => Ok(GeneratedField::BlockNumber),
                            "provenInsertedGer" | "proven_inserted_ger" => Ok(GeneratedField::ProvenInsertedGer),
                            "blockIndex" | "block_index" => Ok(GeneratedField::BlockIndex),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProvenInsertedGerWithBlockNumber;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aggkit.prover.v1.ProvenInsertedGERWithBlockNumber")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProvenInsertedGerWithBlockNumber, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_number__ = None;
                let mut proven_inserted_ger__ = None;
                let mut block_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BlockNumber => {
                            if block_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockNumber"));
                            }
                            block_number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProvenInsertedGer => {
                            if proven_inserted_ger__.is_some() {
                                return Err(serde::de::Error::duplicate_field("provenInsertedGer"));
                            }
                            proven_inserted_ger__ = map_.next_value()?;
                        }
                        GeneratedField::BlockIndex => {
                            if block_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockIndex"));
                            }
                            block_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ProvenInsertedGerWithBlockNumber {
                    block_number: block_number__.unwrap_or_default(),
                    proven_inserted_ger: proven_inserted_ger__,
                    block_index: block_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.ProvenInsertedGERWithBlockNumber", FIELDS, GeneratedVisitor)
    }
}
