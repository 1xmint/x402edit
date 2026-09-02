CREATE TYPE job_state AS ENUM (
  'awaiting_inputs', 'needs_input', 'queued', 'running', 'payment_pending',
  'reconciliation_required', 'ready', 'failed', 'cancelled', 'expired', 'purged'
);

CREATE TABLE jobs (
  id uuid PRIMARY KEY,
  quote_id uuid NOT NULL,
  state job_state NOT NULL,
  phase text NOT NULL,
  capability_digest bytea NOT NULL,
  canonical_request_hash bytea NOT NULL,
  idempotency_key text NOT NULL,
  available_at timestamptz NOT NULL DEFAULT now(),
  lease_owner text,
  lease_expires_at timestamptz,
  fencing_token bigint NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (idempotency_key, canonical_request_hash)
);

CREATE TABLE provider_attempts (
  id uuid PRIMARY KEY,
  job_id uuid NOT NULL REFERENCES jobs(id),
  provider_id text NOT NULL,
  fencing_token bigint NOT NULL,
  provider_request_id text,
  outcome text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE outbox (
  id bigserial PRIMARY KEY,
  aggregate_id uuid NOT NULL,
  event_type text NOT NULL,
  payload jsonb NOT NULL,
  published_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE settlement_intents (
  id uuid PRIMARY KEY,
  job_id uuid NOT NULL UNIQUE REFERENCES jobs(id),
  network text NOT NULL,
  payer_nonce text NOT NULL,
  amount_atomic bigint NOT NULL CHECK (amount_atomic >= 0),
  state text NOT NULL,
  transaction_id text,
  UNIQUE (network, payer_nonce)
);

CREATE TABLE deletion_work (
  id bigserial PRIMARY KEY,
  job_id uuid NOT NULL REFERENCES jobs(id),
  attempt_count integer NOT NULL DEFAULT 0,
  available_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz
);

