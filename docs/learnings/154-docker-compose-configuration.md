# Docker Compose Configuration Learning

## Summary

This document captures the learnings and decisions made during the implementation of Issue #154: Create Docker Compose Configuration.

## Key Decisions

### 1. Gateway Configuration for Docker

The example configuration file sets the gateway bind address to `0.0.0.0` instead of `127.0.0.1`:

```json
"gateway": {
  "bind": {
    "address": "0.0.0.0",
    "ipv6": false
  }
}
```

**Reasoning**: When running in Docker, binding to `127.0.0.1` would only bind to the container's loopback interface, making the service inaccessible from outside the container. Binding to `0.0.0.0` allows the service to accept connections from any interface, including the Docker network bridge.

### 2. Port Configuration

The example config uses port `18789` to match the docker-compose.yml port mapping:

```json
"gateway": {
  "server": {
    "port": 18789
  }
}
```

**Reasoning**: This ensures the containerized service listens on the expected port, which Docker then maps to the host.

### 3. Web UI CORS Configuration

The CORS origins in the example config are updated to include port 18789:

```json
"web_ui": {
  "cors_origins": [
    "http://localhost:18789",
    "http://localhost:5173"
  ]
}
```

**Reasoning**: When accessing the web UI from a Docker container, the browser will make requests to port 18789, so this origin must be allowed.

### 4. Read-Only Config Mount

The docker-compose.yml mounts the config directory as read-only:

```yaml
volumes:
  - ./config:/config:ro
```

**Reasoning**: This prevents the application from modifying configuration files at runtime, which is a security best practice. Configuration should be managed externally.

### 5. Named Volume for Persistent Data

The configuration uses a named volume `aisopod-data` for persistent storage:

```yaml
volumes:
  - aisopod-data:/data
```

**Reasoning**: Named volumes are managed by Docker and persist independently of container lifecycle. They also work across different Docker environments (Docker Desktop, Docker Engine, etc.).

### 6. Environment Variable for Config Path

The config file path is set via environment variable:

```yaml
environment:
  - AISOPOD_CONFIG=/config/aisopod.json
```

**Reasoning**: This allows users to copy the example config file and rename it to `aisopod.json` without modifying the docker-compose.yml. The environment variable provides flexibility for different config file locations.

## Docker Compose Best Practices Applied

1. **Healthchecks**: Container health is monitored with a 30-second interval, allowing time for startup
2. **Restart Policy**: `unless-stopped` ensures automatic restart after failures while allowing manual shutdown
3. **Read-Only Config**: Configuration mounted as read-only for security
4. **Named Volumes**: Persistent data stored in Docker-managed volumes
5. **Non-Root User**: The Dockerfile already defines a non-root user (aisopod:1000), and the compose file doesn't override this

## Future Considerations

### Environment-Specific Configurations

Users may want to create different configurations for development, staging, and production. Consider adding:

- `docker-compose.dev.yml` for development with hot-reloading
- `docker-compose.prod.yml` for production with TLS and security hardening

### Network Configuration

The current setup uses the default bridge network. For more complex deployments, consider:

```yaml
networks:
  aisopod-network:
    driver: bridge
```

### Secrets Management

For production deployments, consider using Docker secrets or external secret management:

```yaml
secrets:
  api_key:
    file: ./secrets/api_key.txt
```

### Multi-Stage Builds

The Dockerfile uses multi-stage builds to keep the runtime image small. The compose file references `build: .` which will use the Dockerfile in the current directory.

## References

- Docker Compose specification: https://docs.docker.com/compose/compose-file/
- Docker best practices: https://docs.docker.com/develop/develop-images/dockerfile_best-practices/
- Issue #153: Dockerfile creation (prerequisite)
- Issue #154: Docker Compose Configuration (current issue)
