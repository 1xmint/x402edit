#![forbid(unsafe_code)]
pub const ALLOWED_FIELDS: &[&str] = &[
    "request_id",
    "job_id",
    "operation",
    "state",
    "phase",
    "provider_id",
    "duration_ms",
    "error_code",
    "amount_atomic",
    "queue_age_ms",
    "fencing_token",
];
pub fn field_allowed(name: &str) -> bool {
    ALLOWED_FIELDS.contains(&name)
}
