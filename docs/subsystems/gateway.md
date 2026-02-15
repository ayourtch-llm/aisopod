# Gateway Subsystem

**Crate:** `aisopod-gateway`

## Overview

The gateway is the central server process — an HTTP + WebSocket server built on Axum
that provides the RPC control plane. It exposes REST endpoints (OpenAI-compatible chat
completions, webhooks, health checks, static UI serving) and a WebSocket endpoint for
JSON-RPC 2.0 style real-time communication with clients and nodes.

## Key Types

- **`GatewayServer`** — Top-level server managing HTTP routes, WebSocket upgrades,
  and graceful shutdown (SIGINT/SIGTERM).
- **`GatewayClient`** — Per-connection state: `conn_id`, `socket`, `presence_key`,
  `client_ip`, `role`, `scopes`, `connected_at`.
- **`RpcMethod` trait** — Handler interface for WebSocket methods
  (`async fn handle(&self, ctx, params) -> Result<Value>`).
- **`ClientRole`** — Either `Operator` (with scopes: admin, read, write, approvals,
  pairing) or `Node`.

## WebSocket Protocol

Follows JSON-RPC 2.0 conventions:
- **Request:** `{ "id", "method", "params" }`
- **Response:** `{ "id", "result" }` or `{ "id", "error": { "code", "message" } }`
- **Broadcast:** `{ "method", "params" }` (no `id`)

24 method namespaces: `agent`, `chat`, `node`, `config`, `skills`, `sessions`,
`system`, `cron`, `models`, `devices`, `approvals`, `updates`.

## Authentication & Rate Limiting

- Auth modes: bearer token, password, device token, Tailscale identity, none (loopback).
- Axum extractors validate HTTP requests; WebSocket handshake validates on upgrade.
- Per-IP sliding-window rate limiting via `DashMap`; returns HTTP 429 with `Retry-After`.

## Event Broadcasting

Uses `tokio::broadcast` channels to push real-time events (presence, health, agent
events, chat events) to connected clients with per-client subscription filtering.

## Dependencies

- **aisopod-config** — `GatewayConfig` (bind address, port, TLS, auth settings).
- **aisopod-agent** — Agent execution via `AgentRunner`.
- **aisopod-session** — Session lookup for chat history methods.
- **aisopod-shared** — Common types (chat envelope, usage aggregates).

## Design Decisions

- **Axum over Actix-Web:** Axum's tower-based middleware ecosystem and tight tokio
  integration match the project's async runtime choice.
- **`DashMap` for client tracking:** Lock-free concurrent map avoids contention on the
  hot path of broadcast fan-out.
- **Embedded static assets:** Web UI files are embedded via `rust-embed` for single-binary
  deployment, with an optional filesystem override for development.
