# Fly.io Deployment Guide

This guide explains how to deploy aisopod to Fly.io with persistent storage and health checks.

## Prerequisites

- A Fly.io account (sign up at [fly.io](https://fly.io))
- The `fly` CLI installed on your local machine
- Docker installed locally (for building the image)

## Installing the Fly CLI

### macOS (Homebrew)

```bash
brew install flyctl
```

### Linux (curl)

```bash
curl -L https://fly.io/install.sh | sh
```

### Windows (Scoop)

```bash
scoop bucket add flyio https://github.com/superfly/scoop-bucket.git
scoop install flyctl
```

### Manual Installation

Download the appropriate binary from the [flyctl releases page](https://github.com/superfly/flyctl/releases).

## Configuration

The `fly.toml` file at the repository root contains all necessary Fly.io configuration:

- App name: `aisopod`
- Primary region: `iad` (Ashburn, VA - US East)
- Machine size: `shared-cpu-1x` with 512MB memory
- HTTP service on port `18789` with HTTPS enforcement
- Persistent volume mount at `/data`
- Health check endpoint at `/health`

## Deployment Steps

### 1. Authenticate with Fly.io

```bash
fly auth login
```

### 2. Launch the Application

Run `fly launch` from the project root directory:

```bash
fly launch
```

This command will:
- Create a new app on Fly.io based on `fly.toml`
- Set up the app name (or prompt you to choose one)
- Configure the primary region
- Prepare for deployment

### 3. Create Persistent Volume

Create a persistent volume for data storage:

```bash
fly volumes create aisopod_data --size 1
```

This creates a 1GB volume named `aisopod_data` that will be mounted to `/data` in the container.

### 4. Set Required Secrets

Configure any necessary environment variables as secrets:

```bash
# Set any API keys or credentials needed by aisopod
fly secrets set AISOPOD_SECRET_KEY="your-secret-key"
fly secrets set AISOPOD_API_TOKEN="your-api-token"
```

Check the [Configuration User Guide](../configuration-user-guide.md) for details on required configuration values.

### 5. Deploy the Application

Deploy to Fly.io:

```bash
fly deploy
```

This will:
- Build the Docker image (using the project's `Dockerfile`)
- Push the image to Fly.io's registry
- Launch the machine with your configured settings

### 6. Verify Deployment

Check the application status:

```bash
fly status
```

View logs:

```bash
fly logs
```

Check volume status:

```bash
fly volumes
```

### 7. Access the Application

Once deployed, access your aisopod instance:

```bash
fly open
```

Or visit the URL shown in the deployment output (typically `https://aisopod.fly.dev`).

## Configuration Reference

### HTTP Service

The `fly.toml` configures an HTTP service with:

- `internal_port = 18789`: The port aisopod listens on
- `force_https = true`: All HTTP requests are redirected to HTTPS
- `auto_stop_machines = "stop"`: Machines stop when idle (cost-saving)
- `auto_start_machines = true`: Machines start automatically on incoming requests
- `min_machines_running = 0`: No machines need to run when idle

### Machine Configuration

- `size = "shared-cpu-1x"`: Basic CPU performance
- `memory = "512mb"`: 512MB RAM

### Health Checks

Health checks run every 30 seconds with a 10-second grace period after startup. The `/health` endpoint should return HTTP 200 for healthy status.

### Environment Variables

- `AISOPOD_CONFIG = "/data/config/aisopod.json"`: Configuration file path on the mounted volume

## Persistent Data

All persistent data is stored in the `/data` volume. This includes:

- Configuration files
- Session data
- Any other state that needs to survive restarts

## Scaling

To scale your deployment:

```bash
# Scale to multiple machines
fly scale count 2 --region iad

# Scale machine size
fly scale size shared-cpu-2x
```

## Updating the Application

When you update the code:

```bash
# Build and deploy in one command
fly deploy

# Or use the GitHub integration for automatic deployments
fly github-actions
```

## Troubleshooting

### Deployment Fails

Check the build logs:

```bash
fly logs --only-builds
```

### Health Checks Failing

Verify the `/health` endpoint is working:

```bash
fly ssh console
curl http://localhost:18789/health
```

### Volume Issues

List volumes:

```bash
fly volumes
```

Attach to a volume:

```bash
fly ssh console --vol-only
```

### Out of Memory

If the application runs out of memory, scale up:

```bash
fly scale memory 1024
```

## Cost Considerations

- Machine costs: Based on the size and uptime
- Volume costs: ~$0.15/GB/month
- Bandwidth: First 100GB/month is free

Use `fly machine list` and `fly volumes` to see your current resources and estimate costs.

## Next Steps

- Review the [Configuration Guide](../configuration-user-guide.md) for setting up aisopod
- Check the [Agents, Channels, Skills Guide](../agents-channels-skills-guide.md) for configuring integrations
- Monitor your deployment using `fly status` and `fly logs`
