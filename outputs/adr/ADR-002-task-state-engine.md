# ADR-002: Task State Engine is the authority

**Status:** Accepted

## Decision

All task records, state transitions, permission checks, board versions, and local history are enforced by one Task State Engine.

## Rationale

The board is a presentation surface and agent adapters are untrusted integration edges. Centralizing authority prevents a client from bypassing confirmation or permissions.

## Consequences

Every client uses the engine API. The engine must be fully testable without the desktop UI.
