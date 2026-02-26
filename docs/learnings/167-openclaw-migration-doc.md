# Issue #167: OpenClaw Migration Documentation - Learning Notes

## Executive Summary

Issue #167 was successfully resolved by creating comprehensive migration documentation that captures all breaking changes between the OpenClaw WebSocket protocol and the Aisopod protocol. The documentation serves as a critical resource for teams maintaining OpenClaw-based clients who need to migrate to Aisopod.

## Learning: Documentation-First Approach

### Key Insight

Creating protocol migration documentation **before** or **alongside** client implementation provides several strategic benefits:

1. **Clear Migration Path**: Clients know exactly what needs to change
2. **Reduced Support Burden**: Common questions are answered in the guide
3. **Testing Foundation**: Documentation becomes the acceptance criteria
4. **Knowledge Transfer**: New team members can understand protocol changes quickly

### Approach Used

For this issue, we:

1. **Studied the existing OpenClaw codebase** to understand method names and parameters
2. **Read the WebSocket protocol specification (Issue #162)** to understand the Aisopod protocol
3. **Examined migration utility code** to understand environment variable mappings
4. **Reviewd existing learnings** for documentation structure and patterns
5. **Created comprehensive migration guide** with examples and troubleshooting

### Documentation Structure

The migration guide is organized as follows:

```
docs/protocol/migration-from-openclaw.md
├── Overview (introduction and key notes)
├── Method Name Changes (OpenClaw → Aisopod table)
├── Parameter Changes (before/after examples)
├── Authentication Changes (headers, tokens, pairing)
├── Environment Variable Renames (mapping table)
├── Protocol Version Mapping (version negotiation)
├── New Features (device pairing, canvas, etc.)
├── Removed Features (what's gone and why)
├── Migration Checklist (client/server testing)
├── Example Migration (code examples)
└── Troubleshooting (common issues and solutions)
```

## Learning: Protocol Method Namespace Changes

### OpenClaw vs Aisopod Namespace Mapping

| Namespace | OpenClaw Pattern | Aisopod Pattern | Changes |
|-----------|------------------|-----------------|---------|
| Node/Device | `claw.*` | `node.*` | Full reorganization |
| Chat | `claw.*` | `chat.*` | Only method rename |
| Canvas | `claw.*` | `canvas.*` | Only method rename |
| Agent | `claw.*` | `agent.*` | Only method rename |
| System | N/A | `system.*` | **NEW** namespace |

### Key Insight: Method Renaming Strategy

The migration required careful analysis of all method calls:

1. **Direct Renames** (simple):
   - `claw.describe` → `node.describe`
   - `claw.execute` → `node.invoke`
   - `claw.send` → `chat.send`

2. **Namespace Changes**:
   - All device-related methods moved to `node.*`
   - Chat methods moved to `chat.*`
   - System methods added new namespace

3. **New Methods** (no OpenClaw equivalent):
   - `node.pair.request`, `node.pair.confirm`, `node.pair.revoke`
   - `approval.request`, `approval.approve`, `approval.deny`, `approval.list`
   - `gateway.subscribe`
   - `system.welcome`, `system.ping`, `system.version`

### Important Finding

The **`claw.execute` → `node.invoke`** change required the most significant parameter restructuring:

**OpenClaw:**
```json
{
  "target": "devices/123/services/camera/takePhoto",
  "args": { "resolution": "high" }
}
```

**Aisopod:**
```json
{
  "service": "camera",
  "method": "takePhoto",
  "params": { "resolution": "high" },
  "timeout_ms": 30000,
  "device_id": "device-123"
}
```

This change:
- Removed the path-based `target` field
- Added structured `service` and `method` fields
- Renamed `args` to `params` (JSON-RPC convention)
- Added `timeout_ms` and `device_id` for better control

## Learning: Authentication Header Evolution

### Header Changes Table

| Header | OpenClaw | Aisopod | Notes |
|--------|----------|---------|-------|
| Authentication | `X-OpenClaw-Token` | `Authorization: Bearer <token>` | Standard HTTP auth |
| Client Info | N/A | `X-Aisopod-Client` | **NEW** required header |
| Device ID | N/A | `X-Aisopod-Device-Id` | **NEW** required header |
| Protocol Version | N/A | `X-Aisopod-Protocol-Version` | **NEW** optional header |

### Key Insight: Device Pairing Requirement

Aisopod introduces **device pairing** as a security enhancement:

**OpenClaw:**
- Devices could invoke capabilities without explicit registration
- Any device with a token could access any endpoint

**Aisopod:**
- Devices must be paired before invoking capabilities
- Pairing flow: `node.pair.request` → user confirmation → `node.pair.confirm`
- Tokens are device-specific and time-limited

This is a **breaking change** that requires client implementation:

```javascript
// OpenClaw: Direct invocation
ws.send(JSON.stringify({
  method: 'claw.execute',
  params: { target: 'devices/me/services/camera/takePhoto' }
}));

// Aisopod: Must pair first
ws.send(JSON.stringify({
  method: 'node.pair.request',
  params: { device_name: 'iPhone', pairing_code: 'ABCD-1234' }
}));

// Then invoke
ws.send(JSON.stringify({
  method: 'node.invoke',
  params: { service: 'camera', method: 'takePhoto', device_id: 'uuid' }
}));
```

## Learning: Environment Variable Naming Convention

### Migration Pattern

All `OPENCLAW_*` environment variables were renamed to `AISOPOD_*` with improved hierarchy:

| OpenClaw | Aisopod | Pattern |
|----------|---------|---------|
| `OPENCLAW_SERVER_PORT` | `AISOPOD_GATEWAY_SERVER_PORT` | `AISOPOD_<component>_<setting>` |
| `OPENCLAW_MODEL_API_KEY` | `AISOPOD_MODELS_PROVIDERS_0_API_KEY` | `AISOPOD_<component>_<index>_<setting>` |
| `OPENCLAW_TOOLS_ENABLED` | `AISOPOD_TOOLS_BASH_ENABLED` | `AISOPOD_<component>_<feature>_<setting>` |

### Key Insight: Component-Based Naming

The new naming follows a component-based pattern:
- `AISOPOD_GATEWAY_*` - Gateway/server settings
- `AISOPOD_MODELS_*` - LLM model/provider settings
- `AISOPOD_TOOLS_*` - Tool configuration
- `AISOPOD_SESSION_*` - Session management
- `AISOPOD_MEMORY_*` - Memory system

This makes it easier to:
- Find related settings
- Understand the system architecture
- Debug configuration issues

## Learning: Protocol Version Negotiation

### Version Headers

**Aisopod:**
```http
X-Aisopod-Protocol-Version: 1.0
```

**OpenClaw:**
- No version header
- No negotiation mechanism
- Clients and servers had to be "lucky" to be compatible

### Key Insight: Graceful Version Handling

The protocol now supports:
1. **Client announces version** in header
2. **Server validates compatibility**
3. **Welcome message includes version** info
4. **Mismatch handling** - connection closed with error

This is a **significant improvement** that enables:
- Backward compatible protocol evolution
- Clear version compatibility guarantees
- Better error messages for incompatible clients

## Learning: New Features Documentation

### Device Pairing Flow

**The most significant new feature** requiring documentation:

1. **Initiation** (`node.pair.request`)
   - Device sends request with pairing code
   - Server generates pairing request

2. **Confirmation** (User action)
   - User confirms via web UI, CLI, or API
   - Server marks as confirmed

3. **Completion** (`node.pair.confirm`)
   - Server sends confirmation response
   - Device receives token
   - Device can now invoke capabilities

**Why this matters:**
- Devices must explicitly register before use
- Provides audit trail of device connections
- Prevents unauthorized device access

### Canvas Protocol

**Purpose:** UI canvas state management

**Methods:**
- `canvas.update` - Push canvas state
- `canvas.interact` - Handle user input

**Use cases:**
- Syncing UI across devices
- Collaborative editing
- Remote UI control

**Key point:** This feature has **no OpenClaw equivalent** - clients must implement from scratch.

## Learning: Removed Features Justification

### Why Features Were Removed

| Feature | OpenClaw | Aisopod | Reason |
|---------|----------|---------|--------|
| Direct device targeting | `devices/123/services/...` | UUID-based | Security: pairing required |
| Unstructured metadata | Ad-hoc in responses | Structured in capability | Consistency |
| Flexible streaming | Any format | Specific methods | Predictability |

### Key Insight: Security Over Convenience

Several OpenClaw features were removed for **security reasons**:

1. **Device Pairing**: Prevents unauthorized device access
2. **Structured Metadata**: Ensures predictable capability discovery
3. **Explicit Versioning**: Catches compatibility issues early

These changes make Aisopod **more secure** but require more work to migrate.

## Learning: Documentation Quality Requirements

### What Makes a Good Migration Guide

1. **Clear Examples**: Before/after code snippets
2. **Complete Coverage**: Every method and parameter
3. **Practical Advice**: Migration checklist and troubleshooting
4. **Visual Aids**: Tables and diagrams
5. **Context**: Why changes were made

### This Guide Includes:

✅ **Method name table** - Every OpenClaw method mapped to Aisopod  
✅ **Parameter examples** - Full JSON before/after for key methods  
✅ **Header comparison** - All auth header changes documented  
✅ **Environment variable table** - Complete migration mapping  
✅ **Version explanation** - How version negotiation works  
✅ **New features section** - Clear explanation of additions  
✅ **Removed features** - What's gone and why  
✅ **Migration checklist** - Client and server verification steps  
✅ **Code examples** - Complete before/after examples  
✅ **Troubleshooting** - Common issues and solutions  

## Learning: Integration with Existing Codebase

### How Documentation Fits

The migration guide complements existing documentation:

1. **WebSocket Protocol Spec (Issue #162)** - The "what" of the protocol
2. **Migration Guide (Issue #167)** - The "how" of migrating
3. **Configuration Migration (Issue #161)** - The "config file" migration

### Dependencies

This documentation **depends on** Issue #162:
- The protocol specification provides the authoritative Aisopod reference
- All method names and parameters must match the spec
- The spec was completed before this migration guide

## Learning: Testing Strategy

### Verification Checklist

When verifying Issue #167, the following was confirmed:

- [x] **Documentation file created** at `docs/protocol/migration-from-openclaw.md`
- [x] **Method name table complete** - every OpenClaw method mapped
- [x] **Parameter changes documented** - with before/after JSON
- [x] **Authentication changes explained** - all headers documented
- [x] **Environment variable mappings** - complete table included
- [x] **Version mapping explained** - how OpenClaw v1 → Aisopod v1
- [x] **New features documented** - device pairing, canvas, etc.
- [x] **Removed features listed** - with reasons
- [x] **Code examples included** - working before/after examples
- [x] **Migration checklist provided** - for both client and server
- [x] **Troubleshooting section** - common issues and solutions
- [x] **Cargo build passes** - `RUSTFLAGS=-Awarnings cargo build`
- [x] **Cargo check passes** - `cargo check --all`
- [x] **File committed** - all changes committed to git

## Learning: Common Pitfalls and Best Practices

### Pitfall #1: Assuming Direct Equivalents

**Mistake:** Assuming `claw.execute` → `node.invoke` is a simple rename

**Reality:** The parameter structure changed significantly:
- `target` field removed
- `service` and `method` fields added
- `args` → `params` rename
- New `timeout_ms` and `device_id` fields

**Solution:** Document all parameter changes explicitly with examples.

### Pitfall #2: Overlooking Authentication Changes

**Mistake:** Only documenting method name changes

**Reality:** Authentication headers completely changed:
- `X-OpenClaw-Token` → `Authorization: Bearer`
- **NEW** `X-Aisopod-Client` required
- **NEW** `X-Aisopod-Device-Id` required
- **NEW** device pairing required for device operations

**Solution:** Create a dedicated authentication section with header tables.

### Pitfall #3: Not Documenting New Features

**Mistake:** Focusing only on breaking changes

**Reality:** New features require implementation:
- Device pairing flow
- Canvas protocol
- Approval workflow
- Welcome message handling

**Solution:** Document new features separately with examples.

## Conclusion

This documentation issue has been successfully resolved with a comprehensive migration guide that:

1. **Documents all method changes** with clear tables
2. **Explains parameter restructuring** with before/after examples
3. **Details authentication header changes** required for connection
4. **Maps environment variables** from OpenClaw to Aisopod
5. **Explains version negotiation** and protocol version mapping
6. **Documents new features** with implementation guidance
7. **Lists removed features** with justification
8. **Provides a migration checklist** for both clients and servers
9. **Includes troubleshooting** for common issues

The guide is designed to help teams migrate from OpenClaw to Aisopod with minimal friction, providing clear, actionable guidance at every step.

### Final Deliverable

**File:** `docs/protocol/migration-from-openclaw.md`  
**Size:** ~17KB, ~450 lines  
**Coverage:** Complete OpenClaw → Aisopod migration documentation

---

*Created: 2026-02-26*  
*Issue: #167*
