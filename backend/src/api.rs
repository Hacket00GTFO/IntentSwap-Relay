use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use futures_util::StreamExt;
use serde_json::json;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tokio::sync::broadcast;
use tracing::{error, warn};
use uuid::Uuid;

use crate::models::{
    CancelIntentRequest, CreateIntentRequest, DbIntentRow, DbRelayerRecord,
    IntentAnalyticsResponse, IntentBroadcastEvent, IntentCreatedResponse, IntentStatus,
    IntentStatusResponse, ListIntentsQuery, RelayerProposalRequest, RelayerRecord, StoredIntent,
};
use crate::signature::{compute_intent_hash_hex, verify_eip712_signature, SignatureConfig};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub signature_config: SignatureConfig,
    pub broadcaster: broadcast::Sender<IntentBroadcastEvent>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/intents", post(create_intent).get(list_intents))
        .route("/intents/{hash}/status", get(intent_status))
        .route("/intents/{hash}/cancel", post(cancel_intent))
        .route("/intents/subscribe", get(subscribe_intents))
        .route("/relayers/propose", post(relayer_propose))
        .route("/relayers", get(list_relayers))
        .route("/analytics/intent/{hash}", get(intent_analytics))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "service": "intent-relay-backend" }))
}

async fn create_intent(
    State(state): State<AppState>,
    Json(payload): Json<CreateIntentRequest>,
) -> Result<Json<IntentCreatedResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now().timestamp();

    if payload.intent.deadline <= now {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "deadline must be in the future" })),
        ));
    }

    if payload.intent.max_relayer_fee_bps > 10_000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "maxRelayerFeeBps cannot exceed 10000" })),
        ));
    }

    if payload.intent.amount_in == "0" || payload.intent.min_amount_out == "0" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "amount values must be greater than zero" })),
        ));
    }

    let is_valid_signature =
        verify_eip712_signature(&state.signature_config, &payload.intent, &payload.signature)
            .map_err(bad_request)?;

    if !is_valid_signature {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid EIP-712 signature" })),
        ));
    }

    let intent_hash = compute_intent_hash_hex(&payload.intent).map_err(bad_request)?;
    let created_at = Utc::now();

    sqlx::query(
        "
        INSERT INTO intents (
            id, intent_hash, maker, token_in, token_out, amount_in, min_amount_out,
            receiver, deadline, nonce, salt, max_relayer_fee_bps, allowed_relayer,
            referral_code, partial_fill_allowed, signature, status, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12, $13,
            $14, $15, $16, $17, $18
        )
        ",
    )
    .bind(Uuid::new_v4())
    .bind(&intent_hash)
    .bind(payload.intent.maker.to_lowercase())
    .bind(payload.intent.token_in.to_lowercase())
    .bind(payload.intent.token_out.to_lowercase())
    .bind(&payload.intent.amount_in)
    .bind(&payload.intent.min_amount_out)
    .bind(payload.intent.receiver.to_lowercase())
    .bind(payload.intent.deadline)
    .bind(payload.intent.nonce as i64)
    .bind(&payload.intent.salt)
    .bind(payload.intent.max_relayer_fee_bps as i32)
    .bind(
        payload
            .intent
            .allowed_relayer
            .as_ref()
            .map(|value| value.to_lowercase()),
    )
    .bind(payload.intent.referral_code.clone())
    .bind(payload.intent.partial_fill_allowed)
    .bind(&payload.signature)
    .bind(IntentStatus::Pending.as_db_str())
    .bind(created_at)
    .execute(&state.db)
    .await
    .map_err(db_error)?;

    let _ = state.broadcaster.send(IntentBroadcastEvent {
        kind: "NEW_INTENT".to_string(),
        intent_hash: intent_hash.clone(),
        status: IntentStatus::Pending,
        maker: payload.intent.maker.clone(),
        token_in: payload.intent.token_in.clone(),
        token_out: payload.intent.token_out.clone(),
        amount_in: payload.intent.amount_in.clone(),
        min_amount_out: payload.intent.min_amount_out.clone(),
        created_at,
    });

    Ok(Json(IntentCreatedResponse {
        intent_hash,
        status: IntentStatus::Pending,
        created_at,
    }))
}

async fn list_intents(
    State(state): State<AppState>,
    Query(query): Query<ListIntentsQuery>,
) -> Result<Json<Vec<StoredIntent>>, (StatusCode, Json<serde_json::Value>)> {
    let maker_filter = query.maker.map(|m| m.to_lowercase());
    let status_filter = query.status;
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let mut builder = QueryBuilder::<Postgres>::new(
        "
        SELECT
            id, intent_hash, maker, token_in, token_out, amount_in, min_amount_out,
            receiver, deadline, nonce, salt, max_relayer_fee_bps, allowed_relayer,
            referral_code, partial_fill_allowed, signature, status, created_at,
            executed_at, executed_by, final_amount_out, execution_tx_hash
        FROM intents
        ",
    );

    let mut has_where = false;
    if let Some(maker) = maker_filter {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("maker = ");
        builder.push_bind(maker);
        has_where = true;
    }

    if let Some(status) = status_filter {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("status = ");
        builder.push_bind(status.as_db_str());
    }

    builder.push(" ORDER BY created_at DESC LIMIT ");
    builder.push_bind(limit as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset as i64);

    let rows: Vec<DbIntentRow> = builder
        .build_query_as()
        .fetch_all(&state.db)
        .await
        .map_err(db_error)?;

    let intents: Vec<StoredIntent> = rows.into_iter().map(StoredIntent::from).collect();
    Ok(Json(intents))
}

async fn intent_status(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<IntentStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let row = sqlx::query_as::<_, DbIntentRow>(
        "
        SELECT
            id, intent_hash, maker, token_in, token_out, amount_in, min_amount_out,
            receiver, deadline, nonce, salt, max_relayer_fee_bps, allowed_relayer,
            referral_code, partial_fill_allowed, signature, status, created_at,
            executed_at, executed_by, final_amount_out, execution_tx_hash
        FROM intents
        WHERE intent_hash = $1
        ",
    )
    .bind(hash)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?;

    let Some(intent_row) = row else {
        return Err(not_found("intent not found"));
    };

    let intent = StoredIntent::from(intent_row);

    Ok(Json(IntentStatusResponse {
        status: intent.status,
        created_at: intent.created_at,
        executed_at: intent.executed_at,
        executed_by: intent.executed_by,
        final_amount_out: intent.final_amount_out,
    }))
}

async fn cancel_intent(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Json(payload): Json<CancelIntentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let row = sqlx::query_as::<_, DbIntentRow>(
        "
        SELECT
            id, intent_hash, maker, token_in, token_out, amount_in, min_amount_out,
            receiver, deadline, nonce, salt, max_relayer_fee_bps, allowed_relayer,
            referral_code, partial_fill_allowed, signature, status, created_at,
            executed_at, executed_by, final_amount_out, execution_tx_hash
        FROM intents
        WHERE intent_hash = $1
        ",
    )
    .bind(&hash)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?;

    let Some(intent_row) = row else {
        return Err(not_found("intent not found"));
    };

    let intent = StoredIntent::from(intent_row);

    if intent.intent.maker.to_lowercase() != payload.maker.to_lowercase() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "maker does not match intent owner" })),
        ));
    }

    if !matches!(intent.status, IntentStatus::Pending) {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "intent is not pending" })),
        ));
    }

    sqlx::query("UPDATE intents SET status = $1 WHERE intent_hash = $2")
        .bind(IntentStatus::Cancelled.as_db_str())
        .bind(&hash)
        .execute(&state.db)
        .await
        .map_err(db_error)?;

    let _ = state.broadcaster.send(IntentBroadcastEvent {
        kind: "INTENT_CANCELLED".to_string(),
        intent_hash: hash.clone(),
        status: IntentStatus::Cancelled,
        maker: intent.intent.maker,
        token_in: intent.intent.token_in,
        token_out: intent.intent.token_out,
        amount_in: intent.intent.amount_in,
        min_amount_out: intent.intent.min_amount_out,
        created_at: intent.created_at,
    });

    Ok(Json(json!({ "cancelled": true, "status": "CANCELLED" })))
}

async fn relayer_propose(
    State(state): State<AppState>,
    Json(payload): Json<RelayerProposalRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let relayer = sqlx::query_as::<_, DbRelayerRecord>(
        "
        SELECT address, name, reputation_score, total_executed, total_volume, is_active
        FROM relayers
        WHERE address = $1
        ",
    )
    .bind(payload.relayer_address.to_lowercase())
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?;

    let Some(relayer) = relayer else {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "relayer is not whitelisted" })),
        ));
    };

    if !relayer.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "relayer is inactive" })),
        ));
    }

    let current_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM intents WHERE intent_hash = $1")
            .bind(&payload.intent_hash)
            .fetch_optional(&state.db)
            .await
            .map_err(db_error)?;

    let Some(current_status) = current_status else {
        return Err(not_found("intent not found"));
    };

    if current_status != IntentStatus::Pending.as_db_str() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "intent is not pending" })),
        ));
    }

    let proposal_id = Uuid::new_v4();

    sqlx::query(
        "
        INSERT INTO relayer_proposals (
            id, intent_hash, relayer_address, proposed_route, expected_output,
            gas_estimate, proposed_fee_bps, signature
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ",
    )
    .bind(proposal_id)
    .bind(&payload.intent_hash)
    .bind(payload.relayer_address.to_lowercase())
    .bind(&payload.proposed_route)
    .bind(&payload.expected_output)
    .bind(&payload.gas_estimate)
    .bind(payload.proposed_fee_bps as i32)
    .bind(&payload.signature)
    .execute(&state.db)
    .await
    .map_err(db_error)?;

    Ok(Json(json!({ "accepted": true, "proposalId": proposal_id })))
}

async fn list_relayers(
    State(state): State<AppState>,
) -> Result<Json<Vec<RelayerRecord>>, (StatusCode, Json<serde_json::Value>)> {
    let rows = sqlx::query_as::<_, DbRelayerRecord>(
        "
        SELECT address, name, reputation_score, total_executed, total_volume, is_active
        FROM relayers
        ORDER BY reputation_score DESC, total_executed DESC
        ",
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_error)?;

    let relayers = rows.into_iter().map(RelayerRecord::from).collect();
    Ok(Json(relayers))
}

async fn intent_analytics(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<IntentAnalyticsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let intent_row = sqlx::query_as::<_, DbIntentRow>(
        "
        SELECT
            id, intent_hash, maker, token_in, token_out, amount_in, min_amount_out,
            receiver, deadline, nonce, salt, max_relayer_fee_bps, allowed_relayer,
            referral_code, partial_fill_allowed, signature, status, created_at,
            executed_at, executed_by, final_amount_out, execution_tx_hash
        FROM intents
        WHERE intent_hash = $1
        ",
    )
    .bind(&hash)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?;

    let Some(intent_row) = intent_row else {
        return Err(not_found("intent not found"));
    };

    let intent = StoredIntent::from(intent_row);

    let proposal_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM relayer_proposals WHERE intent_hash = $1")
            .bind(&hash)
            .fetch_one(&state.db)
            .await
            .map_err(db_error)?;

    Ok(Json(IntentAnalyticsResponse {
        intent_hash: intent.intent_hash,
        status: intent.status,
        relayer_proposals_count: proposal_count.max(0) as usize,
        executed_by: intent.executed_by,
        final_amount_out: intent.final_amount_out,
        created_at: intent.created_at,
        executed_at: intent.executed_at,
    }))
}

async fn subscribe_intents(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state.broadcaster.subscribe()))
}

async fn handle_ws(mut socket: WebSocket, mut rx: broadcast::Receiver<IntentBroadcastEvent>) {
    loop {
        tokio::select! {
            maybe_message = socket.next() => {
                match maybe_message {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        warn!("ws receive error: {err}");
                        break;
                    }
                }
            }
            received = rx.recv() => {
                match received {
                    Ok(event) => {
                        match serde_json::to_string(&event) {
                            Ok(payload) => {
                                if socket.send(Message::Text(payload.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(err) => {
                                warn!("failed to serialize ws payload: {err}");
                            }
                        }
                    }
                    Err(err) => {
                        warn!("ws broadcast error: {err}");
                        break;
                    }
                }
            }
        }
    }
}

fn bad_request(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": error.to_string() })),
    )
}

fn not_found(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::NOT_FOUND, Json(json!({ "error": message })))
}

fn db_error(error: sqlx::Error) -> (StatusCode, Json<serde_json::Value>) {
    match &error {
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "resource already exists" })),
        ),
        _ => {
            error!("database error: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database operation failed" })),
            )
        }
    }
}
