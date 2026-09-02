# x402edit

x402edit is an accountless, agent-native visual production backend. It exposes a
durable HTTP job API for image generation, editing, deterministic composition,
and portable editable artifacts while treating customer content as ephemeral.

The repository is intentionally fail-closed: live provider traffic, non-ZDR
routing, and Base mainnet settlement are disabled until their documented launch
gates have passed.

## Current implementation status

- Versioned Rust domain contracts and state machines.
- HTTP quote/job/status/commit/cancel/ack API foundation.
- PostgreSQL schema, durable lease/outbox primitives, and an in-memory development store.
- Fail-closed RFC 9180 envelope boundary (cryptographic implementation pending review).
- Portable artifact and deterministic-coordinate contracts.
- Capability/privacy-aware provider registry with nine direct adapters.
- TypeScript x402 edge scaffold and Base Sepolia experiment specification.
- CI and deployment scaffolding.

See [docs/api/quickstart.md](docs/api/quickstart.md) for the local flow and
[docs/launch-gates.md](docs/launch-gates.md) for features that cannot safely be
enabled without external credentials, contracts, or review.

## Local development

```bash
cargo test --workspace
cargo run -p x402edit-api
```

The API starts in fail-closed payment mode. For local synthetic testing only,
set `X402EDIT_ENV=development` and `X402EDIT_PAYMENT_MODE=mock`.

## License

Licensed under either Apache-2.0 or MIT, at your option.
