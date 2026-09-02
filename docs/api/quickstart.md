# API quickstart

Run `cargo run -p x402edit-api`, then call `GET /v1/capabilities`. The local service exposes contract and state-machine behavior only. Paid execution is unavailable until the internal payment edge is configured.

Create an immutable five-minute quote with `POST /v1/quotes`, then create a job with `POST /v1/jobs`. The returned 256-bit capability is the only job credential; send it as `Authorization: Bearer ...`. All mutations also require `Idempotency-Key`.

The current in-memory store is for contract testing. It is not production persistence and deliberately does not execute providers or return readable results.

