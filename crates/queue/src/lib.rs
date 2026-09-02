use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use x402edit_domain::JobId;

pub const LEASE_SECONDS: i64 = 60;
pub const HEARTBEAT_SECONDS: i64 = 15;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lease {
    pub job_id: JobId,
    pub worker_id: String,
    pub fencing_token: i64,
    pub expires_at: DateTime<Utc>,
}

impl Lease {
    pub fn renew(&mut self, now: DateTime<Utc>) {
        self.expires_at = now + Duration::seconds(LEASE_SECONDS);
    }

    pub fn permits_commit(&self, token: i64, now: DateTime<Utc>) -> bool {
        token == self.fencing_token && now < self.expires_at
    }
}

pub const CLAIM_SQL: &str = r#"
SELECT id FROM jobs
WHERE state = 'queued' AND available_at <= now()
ORDER BY available_at, created_at
FOR UPDATE SKIP LOCKED
LIMIT $1
"#;
