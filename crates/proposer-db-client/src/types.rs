use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::types::BigDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
pub enum RequestStatus {
    Unrequested = 0,
    WitnessGeneration = 1,
    Execution = 2,
    Prove = 3,
    Complete = 4,
    Relayed = 5,
    Failed = 6,
    Cancelled = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
pub enum RequestType {
    Range = 0,
    Aggregation = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
pub enum RequestMode {
    Real = 0,
    Mock = 1,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OPSuccinctRequest {
    pub id: i64,
    pub status: RequestStatus,
    pub req_type: RequestType,
    pub mode: RequestMode,
    pub start_block: i64,
    pub end_block: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub proof_request_id: Option<Vec<u8>>,
    pub proof_request_time: Option<NaiveDateTime>,
    pub checkpointed_l1_block_number: Option<i64>,
    pub checkpointed_l1_block_hash: Option<Vec<u8>>,
    pub execution_statistics: Value,
    pub witnessgen_duration: Option<i64>,
    pub execution_duration: Option<i64>,
    pub prove_duration: Option<i64>,
    pub range_vkey_commitment: Vec<u8>,
    pub aggregation_vkey_hash: Option<Vec<u8>>,
    pub rollup_config_hash: Vec<u8>,
    pub relay_tx_hash: Option<Vec<u8>>,
    pub proof: Option<Vec<u8>>,
    pub total_nb_transactions: i64,
    pub total_eth_gas_used: i64,
    pub total_l1_fees: BigDecimal,
    pub total_tx_fees: BigDecimal,
    pub l1_chain_id: i64,
    pub l2_chain_id: i64,
    pub contract_address: Option<Vec<u8>>,
    pub prover_address: Option<Vec<u8>>,
    pub l1_head_block_number: Option<i64>,
}
