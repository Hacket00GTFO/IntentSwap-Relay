CREATE TABLE IF NOT EXISTS intents (
    id UUID PRIMARY KEY,
    intent_hash TEXT NOT NULL UNIQUE,
    maker TEXT NOT NULL,
    token_in TEXT NOT NULL,
    token_out TEXT NOT NULL,
    amount_in TEXT NOT NULL,
    min_amount_out TEXT NOT NULL,
    receiver TEXT NOT NULL,
    deadline BIGINT NOT NULL,
    nonce BIGINT NOT NULL,
    salt TEXT NOT NULL,
    max_relayer_fee_bps INTEGER NOT NULL,
    allowed_relayer TEXT,
    referral_code TEXT,
    partial_fill_allowed BOOLEAN NOT NULL,
    signature TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    executed_at TIMESTAMPTZ,
    executed_by TEXT,
    final_amount_out TEXT,
    execution_tx_hash TEXT,
    CONSTRAINT intents_status_check CHECK (status IN ('PENDING', 'EXECUTED', 'EXPIRED', 'CANCELLED'))
);

CREATE INDEX IF NOT EXISTS idx_intents_maker ON intents (maker);
CREATE INDEX IF NOT EXISTS idx_intents_status ON intents (status);
CREATE INDEX IF NOT EXISTS idx_intents_created_at ON intents (created_at DESC);

CREATE TABLE IF NOT EXISTS relayer_proposals (
    id UUID PRIMARY KEY,
    intent_hash TEXT NOT NULL,
    relayer_address TEXT NOT NULL,
    proposed_route TEXT NOT NULL,
    expected_output TEXT NOT NULL,
    gas_estimate TEXT NOT NULL,
    proposed_fee_bps INTEGER NOT NULL,
    signature TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (intent_hash) REFERENCES intents(intent_hash)
);

CREATE INDEX IF NOT EXISTS idx_proposals_intent_hash ON relayer_proposals (intent_hash);

CREATE TABLE IF NOT EXISTS relayers (
    address TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    reputation_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_executed BIGINT NOT NULL DEFAULT 0,
    total_volume TEXT NOT NULL DEFAULT '0',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
