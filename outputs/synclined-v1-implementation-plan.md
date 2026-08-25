# SyncLined V1 Implementation Plan

**Goal:** Build a local, single-device task board where agents propose typed task-state updates and a developer explicitly confirms shared truth.

**Architectural backbone:** The **Task State Engine** and its **versioned Task Access API**. The engine owns task truth; the API is the public, permission-scoped contract through which every client reads or proposes state. No board, agent, or voice client may bypass it.

**Stack:** Tauri 2 desktop shell, React + TypeScript board, Rust local core, SQLite persistence, and a local IPC interface.

**Spec:** [SyncLined V1 Design](synclined-v1-design.md)

## Global constraints

- Single developer and single device only.
- No passive observation, raw transcript synchronization, auto-merging, cross-device sharing, or multi-human collaboration.
- Share only five record types: goal, decision, change, open question, and handoff.
- Agents propose; only the developer confirms, edits, rejects, or resolves state.
- Every shared item must be visible on the persistent board with provenance.
- History remains after revocation; future permissions do not.

## Phases and milestones

Every milestone must be demoable. It ends with a checkpoint: review what was learned, decide whether the architecture changed, update the relevant documentation, and record a new ADR or update a proposed ADR before starting the next milestone.

### Milestone 1 — Backbone vertical slice

**Proves:** The Task State Engine can carry the complete local truth loop across persistence and a separate consumer.

**Exit criteria:** A developer can create a task, an actor can record a typed proposal, the developer can accept or reject it, the complete history persists locally, and an independent reader can resume from the confirmed state. The sequence is covered by an end-to-end test and a live demonstration. No permissions, board subscriptions, conflict detection, sensitivity policy, handoffs, or other extensions enter scope until this loop is smooth.

1. Create a local task with one goal.
2. Record a typed proposal.
3. Accept or reject that proposal as the developer.
4. Persist both task and record history in SQLite.
5. Start a separate board-reader component, load the persisted task, and resume from its confirmed state.
6. Write an end-to-end acceptance test covering create, propose, accept, reject, persist, and resume.
7. Demonstrate the persisted loop with a separate reader process or component.
8. Document the engine API, persisted record shape, and demo steps.
9. Hold an architecture checkpoint before adding the desktop board or agent IPC.

### Milestone 2 — “What just happened?” access contract

**Milestone:** An authorized client can retrieve a compact, versioned, approved task-state summary from the local engine.

1. Define a versioned Task Access API schema, including contract version, task identity, actor session, scopes, and error shape.
2. Expose only engine-backed read and proposal operations through the API.
3. Add a `whatJustHappened` read operation returning the current goal, newly confirmed decisions and changes, open questions, and next step.
4. Enforce the same permission scope at the API boundary that the engine enforces internally.
5. Write contract tests for schema versioning, denied reads, and the compact summary.
6. Define a transport interface independent of the API contract.
7. Implement a minimal encrypted relay transport that forwards approved API messages only; it stores no task state and has no authority.
8. Demonstrate one phone-side client receiving the summary through the relay after an agent proposal is reviewed and confirmed.
9. Document the public contract, transport boundary, and founding laptop-to-phone workflow.
10. Hold an architecture checkpoint before adding the board UI.

**Milestone:** The developer can see all state continuously and turn a proposal into confirmed shared truth.

1. Render confirmed records by type and proposed records in a review queue.
2. Display authoring session and proposal timestamp on every record.
3. Add accept, edit-and-accept, and reject controls.
4. Subscribe the board to local state events so it updates without refresh.
5. Test the end-to-end loop: create task, propose a decision, accept it, and show it as confirmed.
6. Demonstrate live board updates and the review loop.
7. Document board-state semantics and review controls.
8. Hold an architecture checkpoint before adding agent access.

### Milestone 3 — Persistent board and review loop

**Milestone:** A local agent can receive only authorized confirmed slices and submit only authorized typed proposals.

1. Create opaque session identities and a session lifecycle.
2. Add per-task, per-record-type read and propose grants.
3. Expose local IPC endpoints for `readConfirmedState` and `proposeRecord`.
4. Enforce authorization in the Task State Engine, not only in the UI or adapter.
5. Show grants and active sessions on the board.
6. Test denied reads and denied proposals at the engine boundary.
7. Demonstrate a scoped agent session and a denied operation.
8. Document the local session and permission contract.

### Milestone 4 — Agent sessions and permissions

**Milestone:** Conflicts, stale proposals, sensitive releases, and revocation are visible and cannot be bypassed.

1. On proposal, compare its base board version to the current confirmed version; add a stale-state flag when behind.
2. Match proposals against relevant confirmed records and mark possible contradictions as conflicts; never merge automatically.
3. Add a deterministic, initial sensitive-content policy and an approval state for sensitive release.
4. Require developer approval before a sensitive record can be released to another agent or a handoff.
5. Implement immediate session revocation that blocks future reads, subscriptions, and proposals.
6. Test conflict visibility, stale labeling, sensitivity approval, and revocation separately.
7. Demonstrate each safety state on the board.
8. Document the policy, stale-state, and revocation behavior.

### Milestone 5 — Safety controls and curated handoffs

**Milestone:** A developer can review and release a safe handoff snapshot with complete local test coverage of the V1 contract.

1. Build a handoff composer limited to goal, confirmed decisions, open questions, and next steps.
2. Exclude chat logs and unreviewed proposals by construction.
3. Require the same sensitive-field review before release.
4. Add a local acceptance test covering the complete happy path plus conflict, stale-state, and revocation paths.
5. Run unit, integration, UI, lint, type-check, and desktop smoke verification before release.
6. Demonstrate a reviewed, curated handoff.
7. Update the user guide and architecture record index.

## Architecture checkpoints

At the end of every milestone, answer: **Did we learn something that changes the architecture?** If yes, update the plan and add or amend an ADR before continuing. If no, record the decision to continue unchanged.

- **Checkpoint 1:** Does the persisted vertical slice prove the Task State Engine boundary is sufficient for an independent reader to resume a task?
- **Checkpoint 2:** Does the public versioned contract provide enough approved state for a phone-side client to answer “what just happened?”
- **Checkpoint 3:** Does the persistent board remain a simple consumer of the Task Access API?
- **Checkpoint 4:** Do agent permissions belong entirely in the engine, or does the API expose a missing capability boundary?
- **Checkpoint 5:** Do conflict, stale-state, sensitivity, revocation, and curated handoffs require a model change?

## ADR set

- [ADR-001: Local desktop stack](adr/ADR-001-local-desktop-stack.md)
- [ADR-002: Task State Engine as the authority](adr/ADR-002-task-state-engine.md)
- [ADR-003: Explicit proposal and human confirmation](adr/ADR-003-human-confirmation.md)
- [ADR-004: Local-first scope and session identity](adr/ADR-004-local-first-session-identity.md)
- [ADR-005: Typed records, curated handoffs, and no raw chat](adr/ADR-005-typed-records-curated-handoffs.md)
- [ADR-006: Append-only history, conflicts, staleness, and revocation](adr/ADR-006-history-conflicts-revocation.md)
- [ADR-007: Versioned Task Access API](adr/ADR-007-versioned-task-access-api.md)
- [ADR-008: Minimal encrypted relay transport](adr/ADR-008-minimal-encrypted-relay.md)
