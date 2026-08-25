# ADR-006: Append-only history, conflicts, staleness, and revocation

**Status:** Accepted

## Decision

Keep local history after access revocation. Flag conflicting proposals and proposals based on older board state; do not merge them automatically. Revocation blocks future reads, subscriptions, and proposals immediately.

## Rationale

The developer needs legibility without hidden synchronization. A visible "behind" signal is familiar and lightweight, while append-only history supports later audits.

## Consequences

The system cannot retract information already shared. Corrections supersede prior confirmed records instead of deleting them.
