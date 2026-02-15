# Memory System

**Crate:** `aisopod-memory`

## Overview

The memory system (QMD — query-memory database) gives agents persistent, semantic
recall across conversations. It stores facts and context as vector embeddings in
SQLite-Vec, enabling cosine-similarity search to inject relevant memories into
agent prompts automatically.

## Key Types

- **`MemoryStore`** — Core storage: `store()`, `query()`, `delete()`, `list()`.
  Backed by `rusqlite` with the `sqlite-vec` extension loaded.
- **`MemoryEntry`** — Stored fact: id, agent_id, content, embedding (`Vec<f32>`),
  metadata, timestamps.
- **`MemoryMetadata`** — Source (agent/user/system), session key, tags, importance
  score (0.0–1.0), custom JSON.
- **`MemoryMatch`** — Query result: entry plus similarity score.
- **`MemoryQueryOptions`** — Top-K, agent scope, tag filters, importance threshold.
- **`EmbeddingProvider` trait** — Abstract embedding generation (OpenAI
  `text-embedding-3-small`, local via Ollama).

## Storage

Two SQLite structures:

- **`memories` table** — Relational metadata (agent_id, content, source, tags,
  importance, timestamps).
- **`memory_embeddings` virtual table** — `vec0` virtual table storing float
  vectors for cosine similarity search. Default dimension: 1536.

## Query Pipeline

1. Generate embedding for the query string
2. Vector similarity search (top-K nearest neighbors)
3. Filter by agent_id, tags, importance threshold
4. Re-rank results
5. Format matched memories for system prompt injection

## Automatic Memory Management

- Key facts are extracted from conversations post-run for storage.
- Importance scoring combines frequency, recency, and explicit user marking.
- Consolidation merges semantically similar entries.
- Expiration removes stale or low-importance entries.
- Per-agent storage quotas prevent unbounded growth.

## LanceDB Alternative

An optional LanceDB backend is available behind the `lancedb` Cargo feature flag.
It implements the same `MemoryStore` interface with a different storage engine for
more advanced vector operations and larger-scale deployments.

## Dependencies

- **aisopod-config** — `MemoryConfig` (embedding model, dimensions, quotas).
- **aisopod-provider** — Embedding API calls via provider infrastructure.
- **aisopod-session** — Session context for scoping memory extraction.

## Design Decisions

- **SQLite-Vec over a dedicated vector DB:** Keeps the single-file, zero-dependency
  deployment model. `sqlite-vec` is loaded as an extension into the existing rusqlite
  connection.
- **Embedding caching:** Embeddings are stored alongside content to avoid redundant
  API calls on re-indexing or startup.
- **Agent-scoped isolation:** Every memory entry is tagged with `agent_id`, ensuring
  agents cannot read each other's memories unless explicitly shared.
