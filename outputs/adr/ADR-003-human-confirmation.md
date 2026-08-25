# ADR-003: Explicit proposal and human confirmation

**Status:** Accepted

## Decision

Agents create proposals only. The developer alone can confirm, edit and confirm, reject, or resolve task state.

## Rationale

The primary risks are leakage and wrong carryover. Human confirmation makes shared truth inspectable and correctable in flight.

## Consequences

No auto-merge or silent overwrite exists in V1. The review queue is a core product workflow, not optional moderation.
