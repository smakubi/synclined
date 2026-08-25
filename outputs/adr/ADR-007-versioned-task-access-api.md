# ADR-007: Versioned Task Access API

**Status:** Accepted

## Decision

Expose the local Task State Engine through a public, versioned Task Access API. Every client—including the board, coding agent, and phone-side voice client—uses this contract. The API is permission-scoped and is not an alternate source of truth.

## Rationale

The founding workflow is coding on a laptop while asking by voice on a phone what changed. A stable shared contract replaces screenshots and retelling without giving any client authority over approved task state.

## Consequences

The first product-complete path includes a compact `whatJustHappened` response containing the current goal, recent confirmed decisions and changes, open questions, and next step. API requests and responses carry an explicit schema version from the start.
