# ADR-008: Minimal encrypted relay transport

**Status:** Accepted

## Decision

Use a minimal encrypted relay for the first laptop-to-phone demo. Define a transport interface so relay and local-network transports can be exchanged without changing the versioned Task Access API.

## Rationale

Local-network pairing is too situational for the founding workflow. The relay makes the phone-to-laptop path demonstrable while preserving the laptop-local Task State Engine as the sole task authority.

## Consequences

The relay forwards encrypted, authenticated API messages and stores no task state. It cannot confirm proposals, resolve conflicts, or grant permissions. Relay identity and authorization must be scoped to the developer-owned task.
