# ADR-005: Typed records and curated handoffs

**Status:** Accepted

## Decision

V1 accepts only goal, decision, change, open question, and handoff records. A handoff is a reviewed snapshot of goal, confirmed decisions, open questions, and next steps.

## Rationale

Structured task state avoids recreating an unstructured message bus. Excluding raw chat avoids accidental context and secret carryover.

## Consequences

Free-form notes are deferred. Handoffs exclude chat logs and unreviewed proposals by construction.
