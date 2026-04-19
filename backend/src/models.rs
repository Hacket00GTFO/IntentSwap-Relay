use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentPayload {
    pub maker: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: String,
    pub min_amount_out: String,
    pub receiver: String,
    pub deadline: i64,
    pub nonce: u64,
    pub salt: String,
    pub max_relayer_fee_bps: u16,
    pub allowed_relayer: Option<String>,
    pub referral_code: Option<String>,
    pub partial_fill_allowed: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbIntentRow {
    pub id: Uuid,
    pub intent_hash: String,
    pub maker: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: String,
    pub min_amount_out: String,
    pub receiver: String,
    pub deadline: i64,
    pub nonce: i64,
    pub salt: String,
    pub max_relayer_fee_bps: i32,
    pub allowed_relayer: Option<String>,
    pub referral_code: Option<String>,
    pub partial_fill_allowed: bool,
    pub signature: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub executed_by: Option<String>,
    pub final_amount_out: Option<String>,
    pub execution_tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIntentRequest {
    pub intent: IntentPayload,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentStatus {
    Pending,
    Executed,
    Expired,
    Cancelled,
}

impl IntentStatus {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Executed => "EXECUTED",
            Self::Expired => "EXPIRED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn from_db_str(raw: &str) -> Self {
        match raw {
            "EXECUTED" => Self::Executed,
            "EXPIRED" => Self::Expired,
            "CANCELLED" => Self::Cancelled,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredIntent {
    pub id: Uuid,
    pub intent_hash: String,
    pub intent: IntentPayload,
    pub signature: String,
    pub status: IntentStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub executed_by: Option<String>,
    pub final_amount_out: Option<String>,
    pub execution_tx_hash: Option<String>,
}

impl From<DbIntentRow> for StoredIntent {
    fn from(row: DbIntentRow) -> Self {
        Self {
            id: row.id,
            intent_hash: row.intent_hash,
            intent: IntentPayload {
                maker: row.maker,
                token_in: row.token_in,
                token_out: row.token_out,
                amount_in: row.amount_in,
                min_amount_out: row.min_amount_out,
                receiver: row.receiver,
                deadline: row.deadline,
                nonce: row.nonce.max(0) as u64,
                salt: row.salt,
                max_relayer_fee_bps: row.max_relayer_fee_bps.max(0) as u16,
                allowed_relayer: row.allowed_relayer,
                referral_code: row.referral_code,
                partial_fill_allowed: row.partial_fill_allowed,
            },
            signature: row.signature,
            status: IntentStatus::from_db_str(&row.status),
            created_at: row.created_at,
            executed_at: row.executed_at,
            executed_by: row.executed_by,
            final_amount_out: row.final_amount_out,
            execution_tx_hash: row.execution_tx_hash,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentCreatedResponse {
    pub intent_hash: String,
    pub status: IntentStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentStatusResponse {
    pub status: IntentStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub executed_by: Option<String>,
    pub final_amount_out: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIntentsQuery {
    pub maker: Option<String>,
    pub status: Option<IntentStatusFilter>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentStatusFilter {
    Pending,
    Executed,
    Expired,
    Cancelled,
}

impl IntentStatusFilter {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Executed => "EXECUTED",
            Self::Expired => "EXPIRED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelIntentRequest {
    pub maker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayerProposalRequest {
    pub intent_hash: String,
    pub relayer_address: String,
    pub proposed_route: String,
    pub expected_output: String,
    pub gas_estimate: String,
    pub proposed_fee_bps: u16,
    pub signature: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DbRelayerRecord {
    pub address: String,
    pub name: String,
    pub reputation_score: f64,
    pub total_executed: i64,
    pub total_volume: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayerRecord {
    pub address: String,
    pub name: String,
    pub reputation_score: f64,
    pub total_executed: u64,
    pub total_volume: String,
    pub is_active: bool,
}

impl From<DbRelayerRecord> for RelayerRecord {
    fn from(row: DbRelayerRecord) -> Self {
        Self {
            address: row.address,
            name: row.name,
            reputation_score: row.reputation_score,
            total_executed: row.total_executed.max(0) as u64,
            total_volume: row.total_volume,
            is_active: row.is_active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentBroadcastEvent {
    pub kind: String,
    pub intent_hash: String,
    pub status: IntentStatus,
    pub maker: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: String,
    pub min_amount_out: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentAnalyticsResponse {
    pub intent_hash: String,
    pub status: IntentStatus,
    pub relayer_proposals_count: usize,
    pub executed_by: Option<String>,
    pub final_amount_out: Option<String>,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
}
