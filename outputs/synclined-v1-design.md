# SyncLined V1 Design

## Purpose

SyncLined is a local-first, single-developer task board for deliberate AI-agent collaboration. It shares approved task state rather than conversation history, screenshots, files, or passive observations.

The core interaction is version-control-like: agents propose; the developer reviews; accepted updates become shared truth; conflicts remain explicit until the developer resolves them.

## Scope

V1 runs on one developer's device for one task at a time. It supports session-scoped agents, an always-visible memory board, and five typed task-state records:

- Goal
- Decision
- Change
- Open question
- Handoff

It does not observe the developer's environment, synchronize raw chats, auto-merge updates, share between devices, or support multiple human collaborators.

## Roles and authority

The developer is the task owner and final authority. An agent has a session identity and only the permissions the developer grants for that task. Agent display names are not authoritative identifiers; each action carries a session actor reference and provenance that can later support stable identities.

Agents can read granted record types and propose allowed record types. They cannot silently change confirmed state. The developer can accept, edit, reject, or revoke at any time.

## Always-visible board

The board remains visible throughout the task. It shows:

- Confirmed task state, organized by record type
- Proposed updates and their authoring session
- Permission grants and revocations
- Conflicts and stale-state warnings
- Review controls for the developer

This board is the shared workspace. Synchronization is an implementation detail, never hidden behavior.

## Primary flow

1. The developer creates a task and enters an initial goal.
2. The developer connects an agent and grants a minimal read/propose scope.
3. The agent reads only that allowed task-state slice.
4. The agent works in its own environment and submits a typed proposal.
5. The board displays the proposal as unconfirmed.
6. The developer accepts, edits, or rejects it.
7. An accepted update becomes confirmed shared state, available only to agents granted access to its type.

## Policy and sensitivity

Routine proposals appear on the board for review. A policy gate identifies a sensitive proposed share and requires a brief developer confirmation before it is released to another agent or included in a handoff. The developer may exclude or redact the sensitive field.

V1 favors explicit policy rules over passive detection. The product must not silently share sensitive material.

## Conflicts and stale state

A proposal that contradicts confirmed state is shown alongside the relevant confirmed item and flagged as a conflict. SyncLined never merges or replaces either item automatically.

When an agent reads state, the system records the confirmed board version it saw. If that version changes before the agent proposes, its update remains visible but is labeled: "Based on an older task state — review." The developer may accept, edit, reject, or ask the agent to refresh and resubmit.

## Revocation and history

Revoking a session immediately blocks future reads, subscriptions, and proposals within the granted scope. Historical records remain on the board with their provenance; permissions do not. SyncLined cannot retract information already received by an agent, so the interface makes future sharing and handoffs explicit.

## Handoffs

A handoff is a curated snapshot of the current goal, confirmed decisions, open questions, and next steps. It excludes chat logs by design. The developer reviews the exact fields before transfer, including any sensitive-item confirmation or redaction.

## Data model

Each record contains:

- A task-local record identifier and record type
- Content and status: proposed, confirmed, rejected, or superseded
- Authoring session reference and session provenance
- Creation time and the confirmed board version the agent last read
- Optional relationship to a conflicting or superseded record
- Visibility scope and policy-review state

Confirmed state is append-only in V1. Corrections are represented by a new confirmed record that supersedes the prior one, preserving an inspectable local history.

## Acceptance criteria

- A developer can create a local task, add a goal, connect an agent, and grant a limited scope.
- An agent can retrieve only allowed confirmed record types and submit an allowed typed proposal.
- A proposed update is visible on the board before it can become confirmed shared state.
- The developer can accept, edit, or reject a proposal.
- Conflicting and stale proposals are visibly flagged and never auto-merged.
- Revocation immediately stops further sharing for that session while preserving history.
- Handoffs contain only curated task-state fields and require review of sensitive fields.

## Deferred work

Stable verified identity, multi-device synchronization, human collaboration, free-form notes, environmental observation, durable audit export, and automated policy inference are intentionally deferred.
