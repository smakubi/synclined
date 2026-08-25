# ADR-001: Local desktop stack

**Status:** Accepted

## Decision

Use a local desktop shell with a React and TypeScript board, a Rust local core, SQLite persistence, and local IPC. Tauri 2 is the proposed shell.

## Rationale

V1 is intentionally one developer on one device. A desktop-local architecture minimizes network exposure and gives the board a persistent session surface.

## Consequences

The local core can be treated as the future synchronization boundary. Cross-device sharing is deferred and must not be implied by the V1 transport.
