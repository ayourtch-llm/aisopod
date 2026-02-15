# Issue 167: Document Breaking Changes from OpenClaw Protocol

## Summary
Create a migration guide that documents all breaking changes between the OpenClaw protocol and the aisopod protocol, so that developers maintaining OpenClaw-based clients can update their implementations.

## Location
- Crate: N/A (documentation only)
- File: `docs/protocol/migration-from-openclaw.md`

## Current Behavior
No migration documentation exists. Developers familiar with the OpenClaw protocol have no reference for what changed and how to update their code.

## Expected Behavior
A comprehensive migration document covering:

1. **Method name changes** — All renamed RPC methods (e.g. `claw.execute` → `node.invoke`, etc.)
2. **Parameter changes** — Fields that were renamed, added, or removed in request/response objects.
3. **Authentication changes** — New header names, token format changes, device pairing replacing any prior auth mechanism.
4. **Environment variable renames** — Any configuration variables that changed names (e.g. `OPENCLAW_PORT` → `AISOPOD_PORT`).
5. **Protocol version mapping** — How OpenClaw v1 maps to aisopod v1, which features are new, which are dropped.
6. **Removed features** — Any OpenClaw capabilities that are intentionally not carried forward.
7. **New features** — Capabilities in aisopod that have no OpenClaw equivalent (canvas protocol, device pairing, etc.)

## Impact
Without this document, any team migrating from OpenClaw will face unnecessary friction and potential runtime errors from using outdated method names or parameters.

## Suggested Implementation
1. Create the file `docs/protocol/migration-from-openclaw.md`.
2. Use this structure:
   ```markdown
   # Migration Guide: OpenClaw → Aisopod Protocol

   ## Overview
   This guide covers all breaking changes between the OpenClaw WebSocket
   protocol and the Aisopod protocol v1.0.

   ## Method Name Changes
   | OpenClaw Method      | Aisopod Method       | Notes                    |
   |----------------------|----------------------|--------------------------|
   | `claw.execute`       | `node.invoke`        | Params restructured      |
   | `claw.describe`      | `node.describe`      | Added `methods` field    |
   | ...                  | ...                  | ...                      |

   ## Parameter Changes
   ### node.invoke (was claw.execute)
   - Removed: `target` field
   - Added: `service` and `method` fields (replaces single `target`)
   - Changed: `args` renamed to `params`, now accepts arbitrary JSON

   ## Authentication Changes
   - Header `X-OpenClaw-Token` → `Authorization: Bearer <token>`
   - New header: `X-Aisopod-Device-Id` (required for device connections)
   - New header: `X-Aisopod-Client` (required, identifies client app)
   - Device pairing is now required before device-specific operations

   ## Environment Variables
   | OpenClaw Variable    | Aisopod Variable     |
   |----------------------|----------------------|
   | `OPENCLAW_PORT`      | `AISOPOD_PORT`       |
   | `OPENCLAW_SECRET`    | `AISOPOD_JWT_SECRET` |
   | ...                  | ...                  |

   ## Protocol Version Mapping
   - OpenClaw had no formal versioning
   - Aisopod starts at protocol version `1.0`
   - Version is negotiated via `X-Aisopod-Protocol-Version` header

   ## New Features (no OpenClaw equivalent)
   - Device pairing (`node.pair.*`)
   - Canvas protocol (`canvas.*`)
   - Protocol version negotiation
   - Welcome message with server capabilities

   ## Removed Features
   - (List any OpenClaw features intentionally dropped)
   ```
3. Fill in all method mappings by cross-referencing the protocol spec (issue 162) with the OpenClaw source.
4. Have the document reviewed by someone familiar with both protocols.

## Dependencies
- Issue 162 (WebSocket protocol specification — needed as the authoritative aisopod reference)

## Acceptance Criteria
- [ ] `docs/protocol/migration-from-openclaw.md` exists and is complete
- [ ] All renamed methods are listed with old and new names
- [ ] Parameter changes are documented with before/after examples
- [ ] Authentication changes are clearly described
- [ ] Environment variable renames are listed
- [ ] Protocol version mapping is explained
- [ ] New and removed features are documented
- [ ] Document passes review for accuracy

---
*Created: 2026-02-15*
