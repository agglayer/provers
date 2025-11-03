use std::collections::HashMap;

use aggchain_proof_types::{AggchainProofInputs, OptimisticAggchainProofInputs};
use agglayer_interop::types::U256;
use prost::bytes::Bytes;
macro_rules! context_fields {
    ($context:ident, [ $( $key:ident : $value:expr ),* $(,)? ]) => {{
        $(
            context_field!($context, $key: $value);

        )*
    }};
}
macro_rules! context_field {
    ($context: ident, $name:ident: $($data:tt)*) => {
        $context.insert(stringify!($name).to_owned(), Bytes::from($($data)*.to_vec()));
    };
}

macro_rules! int_to_bytes {
    ($val:expr) => {
        Bytes::from($val.to_be_bytes().to_vec())
    };
}

pub trait Contextualize {
    fn context(&self) -> HashMap<String, Bytes>;
}

impl Contextualize for OptimisticAggchainProofInputs {
    fn context(&self) -> HashMap<String, Bytes> {
        self.aggchain_proof_inputs.context()
    }
}

impl Contextualize for AggchainProofInputs {
    fn context(&self) -> HashMap<String, Bytes> {
        let mut context = HashMap::new();

        context_fields!(context, [
            last_proven_block: self.last_proven_block.to_be_bytes(),
            requested_end_block: self.requested_end_block.to_be_bytes(),
            l1_info_tree_root_hash: self.l1_info_tree_root_hash.as_bytes(),
            l1_info_tree_index: self.l1_info_tree_leaf.l1_info_tree_index.to_be_bytes(),
            l1_info_tree_rer: self.l1_info_tree_leaf.rer.as_bytes(),
            l1_info_tree_mer: self.l1_info_tree_leaf.mer.as_bytes(),
            l1_info_tree_ger: self.l1_info_tree_leaf.inner.global_exit_root.as_bytes(),
            l1_info_tree_block_hash: self.l1_info_tree_leaf.inner.block_hash.as_bytes(),
            l1_info_tree_timestamp: self.l1_info_tree_leaf.inner.timestamp.to_be_bytes()
        ]);

        for (name, ger) in self.ger_leaves.iter() {
            context.insert(
                format!("ger/{name}/block_number"),
                int_to_bytes!(ger.block_number),
            );
            context.insert(
                format!("ger/{name}/log_index"),
                int_to_bytes!(ger.log_index),
            );
            context.insert(
                format!("ger/{name}/l1_leaf_index"),
                int_to_bytes!(ger.inserted_ger.l1_leaf.l1_info_tree_index),
            );
        }
        for (i, ibe) in self.imported_bridge_exits.iter().enumerate() {
            context.insert(
                format!("ibe/{i}/block_number"),
                int_to_bytes!(ibe.block_number),
            );
            context.insert(
                format!("ibe/{i}/bridge_exit_hash"),
                Bytes::from(ibe.bridge_exit_hash.0.as_bytes().to_vec()),
            );
            let global_index: U256 = ibe.global_index.into();
            context.insert(
                format!("ibe/{i}/global_index"),
                Bytes::from(
                    global_index
                        .as_le_bytes()
                        .iter()
                        .rev()
                        .copied()
                        .collect::<Vec<_>>(),
                ),
            );
        }

        context
    }
}
