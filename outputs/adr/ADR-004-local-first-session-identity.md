# ADR-004: Local-first scope and session identity

**Status:** Accepted

## Decision

V1 is local to one developer and device. Agents use opaque per-session identities, while record provenance reserves optional stable-principal fields.

## Rationale

This constrains the first release while preventing a future stable identity model from requiring a history rewrite.

## Consequences

Cross-device, multi-user, and verified-identity features require new ADRs and are not implicit extensions of V1.
