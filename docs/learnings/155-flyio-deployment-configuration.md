# Fly.io Deployment Configuration - Learnings

This document captures key learnings from implementing the Fly.io deployment configuration for Issue #155.

## Configuration File Structure

The `fly.toml` file is the primary configuration for Fly.io deployments. Key sections:

### App Definition
```toml
app = "aisopod"
primary_region = "iad"
```
- `app`: Unique name for your application on Fly.io
- `primary_region`: The default region where machines will be created

### Build Configuration
```toml
[build]
  dockerfile = "Dockerfile"
```
- Points to the Dockerfile for building the application image
- Can also specify `context` and `build-target` if needed

### Environment Variables
```toml
[env]
  AISOPOD_CONFIG = "/data/config/aisopod.json"
```
- Application environment variables
- Sensitive values should use `fly secrets set` instead

### HTTP Service
```toml
[http_service]
  internal_port = 18789
  force_https = true
  auto_stop_machines = "stop"
  auto_start_machines = true
  min_machines_running = 0
```
- `internal_port`: Port the application listens on inside the container
- `force_https`: Redirects all HTTP to HTTPS
- `auto_stop_machines`: Stops machines when idle (cost-saving)
- `auto_start_machines`: Automatically starts machines on incoming requests
- `min_machines_running`: Minimum machines to keep running

### Machine Configuration
```toml
[[vm]]
  size = "shared-cpu-1x"
  memory = "512mb"
```
- Defines the VM resources for each machine
- `size`: CPU allocation (shared-cpu-1x, shared-cpu-2x, performance-1x, etc.)
- `memory`: RAM allocation in mb or gb

### Volume Mounts
```toml
[mounts]
  source = "aisopod_data"
  destination = "/data"
```
- Connects a persistent volume to the container
- `source`: Volume name
- `destination`: Mount path in the container

### Health Checks
```toml
[[checks]]
  grace_period = "10s"
  interval = "30s"
  method = "GET"
  path = "/health"
  port = 18789
  timeout = "5s"
  type = "http"
```
- Configures Fly.io's health monitoring
- `grace_period`: Time to wait after start before checking
- `interval`: How often to check
- `path`: Endpoint to check for health

## Deployment Workflow

### Standard Deployment Steps

1. **Install flyctl**: Get the CLI from your package manager or curl
2. **Authenticate**: `fly auth login`
3. **Launch**: `fly launch` - creates the app and configures regions
4. **Create volumes**: `fly volumes create <name> --size 1`
5. **Set secrets**: `fly secrets set <key>=<value>`
6. **Deploy**: `fly deploy`

### Dockerfile Requirements

The Dockerfile must:
- Build the application for release
- Expose the correct port (18789 for aisopod)
- Have a health check endpoint at `/health`
- Run as non-root user for security
- Create the data directory for volume mounts

### Persistent Volumes

Fly.io volumes:
- Are separate from the container filesystem
- Persist data across container restarts
- Can be backed up and restored
- Are regional (must be in the same region as machines)
- Cost: ~$0.15/GB/month

## Common Pitfalls and Solutions

### Machine Stops Too Quickly

If `auto_stop_machines = "stop"` and you need the app always running:

```toml
[http_service]
  min_machines_running = 1
```

### Volume Mount Path Mismatch

Ensure the volume destination matches what the application expects. In our case, `AISOPOD_CONFIG` points to `/data/config/aisopod.json`, so the volume should mount to `/data`.

### Health Check Endpoints

The `/health` endpoint must:
- Return HTTP 200 for healthy status
- Be quick to respond (under the timeout)
- Not require authentication

### Region Selection

Choose regions close to your users:
- `iad` - US East (Ashburn, VA)
- `dfw` - US Central (Dallas, TX)
- `ord` - US East (Chicago, IL)
- `lax` - US West (Los Angeles, CA)
- `nrt` - Asia Pacific (Tokyo)
- `lhr` - Europe (London)

## Cost Optimization

1. **Auto-stop**: Machines stop when idle to save costs
2. **Minimal size**: Start with `shared-cpu-1x` and scale up if needed
3. **Minimal volume**: Start with 1GB and increase as needed
4. **Single region**: Deploy to one region unless global distribution needed

## Verification Commands

After deployment, verify with:

```bash
# Check app status
fly status

# View logs
fly logs

# List machines
fly machine list

# List volumes
fly volumes

# Validate configuration
fly config validate
```

## Security Considerations

1. **Secrets**: Use `fly secrets set` for sensitive data, not environment variables in `fly.toml`
2. **HTTPS**: Enable `force_https = true` to encrypt traffic
3. **Non-root**: Run containers as non-root user
4. **Network**: Consider configuring `[[services]]` for additional network controls

## Next Steps

- Monitor usage and adjust machine size if needed
- Set up alerting for machine failures
- Consider adding secondary regions for redundancy
- Implement CI/CD for automated deployments (see Issue #159)

## Related Issues

- Issue #153: Dockerfile configuration (prerequisite)
- Issue #159: CI/CD release pipeline
- Issue #195: Security deployment documentation
