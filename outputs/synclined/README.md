# SyncLined heartbeat milestone

The local Task State Engine is authoritative. Every client reads approved state through the versioned Task Access API; transports only forward requests.

```mermaid
flowchart LR
  C["Cloud Code"] -->|"propose"| E["Local Task State Engine"]
  H["Developer"] -->|"approve"| E
  P["Phone client"] --> R["Encrypted relay transport"]
  R --> A["Versioned Task Access API"]
  B["Plain board"] --> A
  A -->|"approved state only"| E
  A -->|"whatJustHappened v1"| R
  R --> P
```

## Public heartbeat contract

`whatJustHappened(taskId, schemaVersion, actor)` returns a permission-scoped summary with the current goal and recently confirmed changes. The initial schema version is `v1`.

The `TaskTransport` interface separates delivery from the contract. `LoopbackRelay` is the demo transport; it carries no task state and does not confirm proposals.

## Plain board contract

The board is an API client, not an engine client. It shows the current goal and immutable pending proposals. Its accept, reject, and edit-then-accept actions are routed through `TaskAccessApi`; it holds no parallel task state.

## Demo

Run `cargo test --test relay_heartbeat` to demonstrate: propose, approve, then retrieve the versioned heartbeat through the relay-backed API.
