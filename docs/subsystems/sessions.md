# Session Management Subsystem

**Crate:** `aisopod-session`

## Overview

The session subsystem provides persistent storage for AI agent conversations.
Sessions are keyed by a composite of agent, channel, account, and peer, ensuring
multi-agent isolation. Message history is stored in SQLite with efficient pagination
and integrated compaction support.

## Key Types

- **`SessionStore`** — Core storage backed by `rusqlite`. Methods: `get_or_create()`,
  `list()`, `get_history()`, `append_messages()`, `patch()`, `delete()`.
- **`SessionKey`** — Composite key: `agent_id`, `channel`, `account_id`, `peer_kind`
  (dm/group/channel), `peer_id`.
- **`Session`** — State record: key, timestamps, message count, token usage,
  metadata (title, last model, compaction count), status.
- **`StoredMessage`** — Persisted message: id, role, content, tool calls,
  timestamp, token count, metadata.
- **`SessionStatus`** — Enum: `Active`, `Idle`, `Compacted`, `Archived`.

## Storage Backend

SQLite via `rusqlite` with two core tables:

- **`sessions`** — Indexed by composite key; stores metadata and aggregate stats.
- **`messages`** — JSON blob content with foreign key to sessions; indexed by
  `(session_key, timestamp)` for efficient history retrieval.

Messages are stored as JSON for flexibility with evolving content schemas.

## Session Routing

- DMs collapse to the agent's `main` session.
- Groups are isolated by peer ID (guild/group/channel ID).
- Agent bindings determine which agent owns which channel/peer combination.

## Compaction Integration

After agent runs, the engine may trigger compaction. The session store tracks
compaction count and preserves summaries. Compaction strategies (summary, hard
clear) operate on the stored message list and update it in place.

## Dependencies

- **aisopod-config** — `SessionConfig` (compaction thresholds, history limits).
- **aisopod-shared** — Common message types and usage aggregation.

## Design Decisions

- **SQLite over Postgres:** Single-file embedded database suits the self-hosted,
  single-process deployment model. No external database dependency.
- **JSON blob messages:** Avoids rigid column schemas for message content, making
  it easy to evolve the `StoredMessage` structure without migrations.
- **Composite session key:** Encodes routing semantics directly into the key,
  making multi-agent isolation a natural property of the data model rather than
  a query-time filter.
