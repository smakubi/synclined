# ADR-004: Local-first scope and session identity

**Status:** Accepted

## Decision

V1 has one developer-owned local Task State Engine, accessible by that developer's approved clients through a versioned Task Access API. Agents use opaque per-session identities, while record provenance reserves optional stable-principal fields.

## Rationale

This keeps the engine local and authoritative while supporting the founding laptop-to-phone workflow without making the phone an independent source of truth.

## Consequences

Multi-user collaboration and verified identity require new ADRs. Approved clients consume the same task-access contract regardless of device.
