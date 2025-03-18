// @generated
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
                    start_block: start_block__.unwrap_or_default(),
                    max_end_block: max_end_block__.unwrap_or_default(),
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
        if self.imported_bridge_exit.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.ImportedBridgeExitWithBlockNumber", len)?;
        if self.block_number != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockNumber", ToString::to_string(&self.block_number).as_str())?;
        }
        if let Some(v) = self.imported_bridge_exit.as_ref() {
            struct_ser.serialize_field("importedBridgeExit", v)?;
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
            "imported_bridge_exit",
            "importedBridgeExit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockNumber,
            ImportedBridgeExit,
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
                            "importedBridgeExit" | "imported_bridge_exit" => Ok(GeneratedField::ImportedBridgeExit),
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
                let mut imported_bridge_exit__ = None;
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
                        GeneratedField::ImportedBridgeExit => {
                            if imported_bridge_exit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importedBridgeExit"));
                            }
                            imported_bridge_exit__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ImportedBridgeExitWithBlockNumber {
                    block_number: block_number__.unwrap_or_default(),
                    imported_bridge_exit: imported_bridge_exit__,
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
        let mut struct_ser = serializer.serialize_struct("aggkit.prover.v1.ProvenInsertedGERWithBlockNumber", len)?;
        if self.block_number != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockNumber", ToString::to_string(&self.block_number).as_str())?;
        }
        if let Some(v) = self.proven_inserted_ger.as_ref() {
            struct_ser.serialize_field("provenInsertedGer", v)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockNumber,
            ProvenInsertedGer,
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
                    }
                }
                Ok(ProvenInsertedGerWithBlockNumber {
                    block_number: block_number__.unwrap_or_default(),
                    proven_inserted_ger: proven_inserted_ger__,
                })
            }
        }
        deserializer.deserialize_struct("aggkit.prover.v1.ProvenInsertedGERWithBlockNumber", FIELDS, GeneratedVisitor)
    }
}
