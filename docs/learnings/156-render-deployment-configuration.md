# Render Deployment Configuration - Learnings

This document captures key learnings from implementing the Render deployment configuration for Issue #156.

## Configuration File Structure

The `render.yaml` file defines the infrastructure for Render's Blueprint deployment system. Key sections:

### Service Definition
```yaml
services:
  - type: web
    name: aisopod
    runtime: docker
    dockerfilePath: ./Dockerfile
    plan: starter
```
- `type`: Service type (web, worker, etc.)
- `name`: Unique service name within the blueprint
- `runtime`: Runtime type (docker, static, etc.)
- `dockerfilePath`: Path to the Dockerfile relative to repo root
- `plan`: Instance plan (starter, basic, standard, plus)

### Health Checks
```yaml
healthCheckPath: /health
```
- Path for Render's health monitoring
- Must return HTTP 200 for healthy status
- Check interval: Every 30 seconds by default
- Timeout: 10 seconds

### Environment Variables
```yaml
envVars:
  - key: AISOPOD_CONFIG
    value: /data/config/aisopod.json
  - key: AISOPOD_BIND_ADDRESS
    value: "0.0.0.0"
  - key: AISOPOD_PORT
    value: "18789"
```
- Define environment variables as a list
- `key`: Variable name
- `value`: Variable value (must be a string)
- Sensitive values should be set via Render dashboard after deployment

### Persistent Disk
```yaml
disk:
  name: aisopod-data
  mountPath: /data
  sizeGB: 1
```
- `name`: Unique disk name within the blueprint
- `mountPath`: Path where disk is mounted in the container
- `sizeGB`: Disk size in gigabytes

## Deployment Workflow

### Standard Deployment Steps

1. **Connect Repository**: Fork or connect the repository to Render
2. **Create Blueprint**: Click "New +" → "Blueprint Instance"
3. **Select Repository**: Choose the aisopod repository
4. **Automatic Detection**: Render reads `render.yaml` automatically
5. **Review Configuration**: Verify service settings
6. **Deploy**: Render builds and deploys the service
7. **Monitor**: Watch logs and metrics in the dashboard

### Repository Connection Options

#### Option 1: Fork the Repository
**Pros:**
- Full control over configuration changes
- Can customize the application code
- Can contribute changes back via PR

**Cons:**
- Must manually sync with upstream
- Changes need to be managed separately

#### Option 2: Connect Original Repository
**Pros:**
- Follows official changes automatically
- Minimal maintenance

**Cons:**
- Limited to default configuration
- Cannot customize without forking

### Dockerfile Requirements

The Dockerfile must:
- Build the application for production
- Expose the correct port (18789 for aisopod)
- Have a health check endpoint at `/health`
- Run as non-root user for security
- Create the data directory for volume mounts
- Be compatible with Render's Docker runtime

### Health Check Endpoint

The `/health` endpoint must:
- Return HTTP 200 for healthy status
- Be quick to respond (under the timeout)
- Not require authentication
- Indicate the application is ready to serve requests

## Common Pitfalls and Solutions

### Build Failures

**Problem**: Docker build fails during deployment

**Solutions:**
- Verify Dockerfile exists at the specified path
- Check that all build dependencies are available
- Test the build locally first: `docker build -t aisopod .`
- Ensure the Cargo.toml has correct workspace configuration

### Environment Variable Not Applied

**Problem**: Environment variables don't seem to take effect

**Solutions:**
- Verify variable names match exactly (case-sensitive)
- Check for typos in `render.yaml`
- Ensure variables are at the correct indentation level
- Restart the service after changing variables

### Health Check Failing

**Problem**: Service shows as unhealthy in the dashboard

**Solutions:**
- Test the `/health` endpoint manually
- Check logs for startup errors
- Verify the application listens on the correct port
- Ensure no authentication is required for the health endpoint

### Disk Mount Issues

**Problem**: Disk not accessible at expected path

**Solutions:**
- Verify `mountPath` matches application expectations
- Check disk permissions in the container
- Ensure the application has write permissions to the mount path
- Verify disk was created successfully in the Render dashboard

### Port Binding Issues

**Problem**: Service cannot bind to the specified port

**Solutions:**
- Verify `AISOPOD_PORT` matches the exposed port in Dockerfile
- Ensure the application binds to `0.0.0.0` (all interfaces)
- Check that the port is not already in use by another process
- Verify no firewall rules are blocking the port

## Environment Variables Reference

### Required Variables (from render.yaml)

| Variable | Default Value | Description |
|----------|--------------|-------------|
| `AISOPOD_CONFIG` | `/data/config/aisopod.json` | Path to configuration file |
| `AISOPOD_BIND_ADDRESS` | `0.0.0.0` | Server bind address |
| `AISOPOD_PORT` | `18789` | Server port |

### Optional Variables (set via dashboard)

| Variable | Description | Example |
|----------|-------------|---------|
| `AISOPOD_PROVIDERS_0_API_KEY` | Provider API key | `sk-...` |
| `AISOPOD_SECRET_KEY` | Application secret key | Random string |
| `AISOPOD_AUTH_PROVIDERS` | Authentication providers | `token,password` |

## Security Considerations

1. **Secrets Management**: Use the Render dashboard to set sensitive values, not `render.yaml`
2. **HTTPS**: Render automatically provides HTTPS for custom domains
3. **Non-root**: The Dockerfile runs as non-root user (user ID 1000)
4. **Network**: Render handles network security groups automatically

## Cost Optimization

1. **Starter Plan**: Free tier available with limitations
2. **Auto-scaling**: Render can scale based on traffic
3. **Minimal Disk**: Start with 1GB and increase as needed
4. **Monitoring**: Monitor usage to optimize costs

## Monitoring and Maintenance

### Dashboard Navigation

1. **Service Overview**: View status, logs, metrics
2. **Environment Tab**: Manage environment variables
3. **Settings Tab**: Change plan, add custom domain
4. **Logs Tab**: View application and system logs
5. **Metrics Tab**: CPU, memory, request statistics

### Log Access

- View real-time logs in the dashboard
- Use search and filter options
- Export logs for analysis
- Set up log streaming to external services

### Metrics Monitoring

- **CPU Usage**: Track resource utilization
- **Memory Usage**: Monitor for leaks or overuse
- **Request Count**: Track traffic patterns
- **Response Time**: Measure performance

## Integration with Other Services

### Docker Compose (Local Development)

For local testing, use the matching configuration from `docker-compose.yml`:

```yaml
ports:
  - "18789:18789"
volumes:
  - ./data:/data
environment:
  - AISOPOD_CONFIG=/data/config/aisopod.json
  - AISOPOD_BIND_ADDRESS=0.0.0.0
  - AISOPOD_PORT=18789
```

### CI/CD Pipeline

For automated deployments, see Issue #159 for GitHub Actions configuration.

## Verification Commands

After deployment, verify with:

```bash
# Test health endpoint
curl https://your-instance.onrender.com/health

# View logs (via dashboard or CLI)
curl -s https://your-instance.onrender.com/logs

# Verify environment
curl https://your-instance.onrender.com/config
```

## Comparison with Other Platforms

### Fly.io vs Render

| Feature | Fly.io | Render |
|---------|--------|--------|
| Configuration | `fly.toml` | `render.yaml` |
| Deployment | `fly deploy` | Git push / Blueprint |
| Scaling | Manual | Automatic |
| Pricing | Pay per use | Free tier + plans |
| Region Selection | Multiple regions | Regional clusters |
| Persistence | Volumes | Disks |

### Key Differences

1. **Deployment Method**:
   - Fly.io: CLI-based (`fly deploy`)
   - Render: Git-based (Blueprint from repo)

2. **Scaling**:
   - Fly.io: Manual or auto-scale with machines
   - Render: Automatic scaling based on traffic

3. **Pricing**:
   - Fly.io: Pay per machine hour and resources
   - Render: Free tier with usage-based pricing

4. **Configuration**:
   - Fly.io: TOML format
   - Render: YAML format

## Related Issues

- Issue #153: Dockerfile configuration (prerequisite)
- Issue #154: Docker Compose configuration
- Issue #155: Fly.io deployment configuration
- Issue #159: CI/CD release pipeline
- Issue #195: Security deployment documentation

## References

- [Render Documentation](https://render.com/docs)
- [Render Blueprint Reference](https://render.com/docs/blueprint-spec)
- [Dockerfile Best Practices](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)
- [Render Pricing](https://render.com/pricing)

## Version History

- 2026-02-25: Initial version for Issue #156
