use sqlx::{PgPool, Row};

use crate::{
    error::Error,
    types::{OPSuccinctRequest, RequestStatus, RequestType},
};

#[derive(Clone)]
pub struct ProposerDBClient {
    pool: PgPool,
}

impl ProposerDBClient {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn insert_request(&self, request: &OPSuccinctRequest) -> Result<i64, Error> {
        let id = sqlx::query(
            r#"
            INSERT INTO requests (
                status, req_type, mode, start_block, end_block, created_at,
                updated_at, proof_request_id, proof_request_time,
                checkpointed_l1_block_number, checkpointed_l1_block_hash,
                execution_statistics, witnessgen_duration, execution_duration,
                prove_duration, range_vkey_commitment, aggregation_vkey_hash,
                rollup_config_hash, relay_tx_hash, proof, total_nb_transactions,
                total_eth_gas_used, total_l1_fees, total_tx_fees, l1_chain_id,
                l2_chain_id, contract_address, prover_address, l1_head_block_number
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29
            )
            RETURNING id
            "#,
        )
        .bind(request.status as i32)
        .bind(request.req_type as i32)
        .bind(request.mode as i32)
        .bind(request.start_block)
        .bind(request.end_block)
        .bind(request.created_at)
        .bind(request.updated_at)
        .bind(request.proof_request_id.as_ref().map(|v| v.as_slice()))
        .bind(request.proof_request_time)
        .bind(request.checkpointed_l1_block_number)
        .bind(request.checkpointed_l1_block_hash.as_ref().map(|v| v.as_slice()))
        .bind(&request.execution_statistics)
        .bind(request.witnessgen_duration)
        .bind(request.execution_duration)
        .bind(request.prove_duration)
        .bind(&request.range_vkey_commitment)
        .bind(request.aggregation_vkey_hash.as_ref().map(|v| v.as_slice()))
        .bind(&request.rollup_config_hash)
        .bind(request.relay_tx_hash.as_ref().map(|v| v.as_slice()))
        .bind(request.proof.as_ref().map(|v| v.as_slice()))
        .bind(request.total_nb_transactions)
        .bind(request.total_eth_gas_used)
        .bind(&request.total_l1_fees)
        .bind(&request.total_tx_fees)
        .bind(request.l1_chain_id)
        .bind(request.l2_chain_id)
        .bind(request.contract_address.as_ref().map(|v| v.as_slice()))
        .bind(request.prover_address.as_ref().map(|v| v.as_slice()))
        .bind(request.l1_head_block_number)
        .fetch_one(&self.pool)
        .await?
        .try_get("id")?;

        Ok(id)
    }

    pub async fn get_consecutive_complete_range_proofs(
        &self,
        start_block: i64,
        end_block: i64,
        range_vkey_commitment: &[u8],
        rollup_config_hash: &[u8],
        l1_chain_id: i64,
        l2_chain_id: i64,
    ) -> Result<Vec<OPSuccinctRequest>, Error> {
        let requests = sqlx::query_as::<_, OPSuccinctRequest>(
            r#"
            SELECT * FROM requests
            WHERE range_vkey_commitment = $1
              AND rollup_config_hash = $2
              AND status = $3
              AND req_type = $4
              AND start_block >= $5
              AND end_block <= $6
              AND l1_chain_id = $7
              AND l2_chain_id = $8
            ORDER BY start_block ASC
            "#,
        )
        .bind(range_vkey_commitment)
        .bind(rollup_config_hash)
        .bind(RequestStatus::Complete as i32)
        .bind(RequestType::Range as i32)
        .bind(start_block)
        .bind(end_block)
        .bind(l1_chain_id)
        .bind(l2_chain_id)
        .fetch_all(&self.pool)
        .await?;

        if requests.is_empty() {
            return Err(Error::NoRangeProofsFound);
        }

        Ok(requests)
    }

    pub async fn get_agg_proof_by_id(&self, proof_id: i64) -> Result<Vec<u8>, Error> {
        let row = sqlx::query(
            r#"
            SELECT proof FROM requests WHERE id = $1
            "#,
        )
        .bind(proof_id)
        .fetch_one(&self.pool)
        .await?;

        let proof: Option<Vec<u8>> = row.try_get("proof")?;
        proof.ok_or(Error::ProofNotFound)
    }
}
