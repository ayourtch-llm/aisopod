# Issue 155: Create Fly.io Deployment Configuration

## Summary
Create a `fly.toml` configuration file for deploying aisopod to Fly.io with persistent volume support, health checks, and HTTP service configuration on port 18789.

## Location
- Crate: `aisopod` (workspace root)
- File: `fly.toml`

## Current Behavior
No Fly.io deployment configuration exists. Users cannot deploy aisopod to Fly.io without manually creating configuration.

## Expected Behavior
A `fly.toml` at the repo root that defines the Fly.io app configuration including app name, region, machine size, persistent volume mount, health checks, and HTTP service settings.

## Impact
Enables one-command cloud deployment to Fly.io, making aisopod accessible to users who prefer managed hosting without maintaining their own infrastructure.

## Suggested Implementation

Create `fly.toml` at the repository root:

```toml
app = "aisopod"
primary_region = "iad"

[build]
  dockerfile = "Dockerfile"

[env]
  AISOPOD_CONFIG = "/data/config/aisopod.json"

[http_service]
  internal_port = 18789
  force_https = true
  auto_stop_machines = "stop"
  auto_start_machines = true
  min_machines_running = 0

[[vm]]
  size = "shared-cpu-1x"
  memory = "512mb"

[mounts]
  source = "aisopod_data"
  destination = "/data"

[[checks]]
  grace_period = "10s"
  interval = "30s"
  method = "GET"
  path = "/health"
  port = 18789
  timeout = "5s"
  type = "http"
```

Also create a deployment guide at `docs/deployment/flyio.md` covering:
1. Installing the `fly` CLI.
2. Running `fly launch` and `fly deploy`.
3. Creating the persistent volume with `fly volumes create aisopod_data --size 1`.
4. Setting secrets with `fly secrets set`.

## Dependencies
- Issue 153 (Dockerfile)

## Acceptance Criteria
- [ ] `fly.toml` is valid and passes `fly config validate`
- [ ] Health check is configured on `/health` endpoint at port 18789
- [ ] Persistent volume mounts to `/data`
- [ ] HTTP service is configured with HTTPS enforcement
- [ ] Deployment guide is documented in `docs/deployment/flyio.md`
- [ ] Machine size and region are sensible defaults

---
*Created: 2026-02-15*
