# Issue 156: Create Render Deployment Configuration

## Summary
Create a `render.yaml` blueprint file that defines a Docker-based web service for aisopod on Render, with a persistent disk, environment variable configuration, and health check path.

## Location
- Crate: `aisopod` (workspace root)
- File: `render.yaml`

## Current Behavior
No Render deployment configuration exists. Users cannot use Render's blueprint-based deployment for aisopod.

## Expected Behavior
A `render.yaml` at the repo root that defines a valid Render blueprint with a Docker web service, 1 GB persistent disk, environment variables, and health check configuration.

## Impact
Enables push-button deployment to Render, giving users another managed hosting option alongside Fly.io.

## Suggested Implementation

Create `render.yaml` at the repository root:

```yaml
services:
  - type: web
    name: aisopod
    runtime: docker
    dockerfilePath: ./Dockerfile
    plan: starter
    healthCheckPath: /health
    envVars:
      - key: AISOPOD_CONFIG
        value: /data/config/aisopod.json
      - key: AISOPOD_BIND_ADDRESS
        value: "0.0.0.0"
      - key: AISOPOD_PORT
        value: "18789"
    disk:
      name: aisopod-data
      mountPath: /data
      sizeGB: 1
```

Also create a deployment guide at `docs/deployment/render.md` covering:
1. Forking or connecting the repository to Render.
2. Using the "New Blueprint Instance" flow.
3. Configuring environment variables for API keys.
4. Monitoring the service via Render dashboard.

## Dependencies
- Issue 153 (Dockerfile)

## Acceptance Criteria
- [x] `render.yaml` is a valid Render blueprint (passes Render's schema validation)
- [x] Service is defined as a Docker web service
- [x] Persistent disk of 1 GB is configured at `/data`
- [x] Health check path is set to `/health`
- [x] Environment variables are properly configured
- [x] Deployment guide is documented in `docs/deployment/render.md`

## Resolution
Created render.yaml at repository root with complete Render blueprint configuration:
- Docker web service defined with `dockerfilePath: ./Dockerfile` and `plan: starter`
- Health check path set to `/health`
- Environment variables configured: `AISOPOD_CONFIG`, `AISOPOD_BIND_ADDRESS`, `AISOPOD_PORT`
- Persistent disk of 1GB configured at `/data`

Created `docs/deployment/render.md` comprehensive deployment guide covering:
- Repository connection to Render
- Blueprint deployment flow
- Environment variable configuration for API keys
- Service monitoring via Render dashboard
- Troubleshooting common issues

Verification completed:
- `cargo build` passes without errors
- `cargo test` passes without errors

All changes committed in two commits.

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
