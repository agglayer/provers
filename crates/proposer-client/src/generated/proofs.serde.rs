// @generated
impl serde::Serialize for AggProofRequest {
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
        if self.l1_block_number != 0 {
            len += 1;
        }
        if !self.l1_block_hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("proofs.AggProofRequest", len)?;
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
        if self.l1_block_number != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("l1BlockNumber", ToString::to_string(&self.l1_block_number).as_str())?;
        }
        if !self.l1_block_hash.is_empty() {
            struct_ser.serialize_field("l1BlockHash", &self.l1_block_hash)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AggProofRequest {
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
            "l1_block_number",
            "l1BlockNumber",
            "l1_block_hash",
            "l1BlockHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LastProvenBlock,
            RequestedEndBlock,
            L1BlockNumber,
            L1BlockHash,
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
                            "l1BlockNumber" | "l1_block_number" => Ok(GeneratedField::L1BlockNumber),
                            "l1BlockHash" | "l1_block_hash" => Ok(GeneratedField::L1BlockHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AggProofRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct proofs.AggProofRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AggProofRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut last_proven_block__ = None;
                let mut requested_end_block__ = None;
                let mut l1_block_number__ = None;
                let mut l1_block_hash__ = None;
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
                        GeneratedField::L1BlockNumber => {
                            if l1_block_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1BlockNumber"));
                            }
                            l1_block_number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::L1BlockHash => {
                            if l1_block_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("l1BlockHash"));
                            }
                            l1_block_hash__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(AggProofRequest {
                    last_proven_block: last_proven_block__.unwrap_or_default(),
                    requested_end_block: requested_end_block__.unwrap_or_default(),
                    l1_block_number: l1_block_number__.unwrap_or_default(),
                    l1_block_hash: l1_block_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("proofs.AggProofRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AggProofResponse {
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
        if self.end_block != 0 {
            len += 1;
        }
        if !self.proof_request_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("proofs.AggProofResponse", len)?;
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
        if !self.proof_request_id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofRequestId", pbjson::private::base64::encode(&self.proof_request_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AggProofResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "last_proven_block",
            "lastProvenBlock",
            "end_block",
            "endBlock",
            "proof_request_id",
            "proofRequestId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LastProvenBlock,
            EndBlock,
            ProofRequestId,
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
                            "endBlock" | "end_block" => Ok(GeneratedField::EndBlock),
                            "proofRequestId" | "proof_request_id" => Ok(GeneratedField::ProofRequestId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AggProofResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct proofs.AggProofResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AggProofResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut last_proven_block__ = None;
                let mut end_block__ = None;
                let mut proof_request_id__ = None;
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
                        GeneratedField::EndBlock => {
                            if end_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endBlock"));
                            }
                            end_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofRequestId => {
                            if proof_request_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofRequestId"));
                            }
                            proof_request_id__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(AggProofResponse {
                    last_proven_block: last_proven_block__.unwrap_or_default(),
                    end_block: end_block__.unwrap_or_default(),
                    proof_request_id: proof_request_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("proofs.AggProofResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetMockProofRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proof_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("proofs.GetMockProofRequest", len)?;
        if self.proof_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofId", ToString::to_string(&self.proof_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetMockProofRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proof_id",
            "proofId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProofId,
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
                            "proofId" | "proof_id" => Ok(GeneratedField::ProofId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetMockProofRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct proofs.GetMockProofRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetMockProofRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ProofId => {
                            if proof_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofId"));
                            }
                            proof_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GetMockProofRequest {
                    proof_id: proof_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("proofs.GetMockProofRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetMockProofResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proof.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("proofs.GetMockProofResponse", len)?;
        if !self.proof.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetMockProofResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proof,
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
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetMockProofResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct proofs.GetMockProofResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetMockProofResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GetMockProofResponse {
                    proof: proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("proofs.GetMockProofResponse", FIELDS, GeneratedVisitor)
    }
}
