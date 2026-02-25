# Render Deployment Guide

This guide explains how to deploy aisopod to Render using the blueprint-based deployment system.

## Prerequisites

- A Render account (sign up at [render.com](https://render.com))
- A GitHub, GitLab, or Bitbucket account (for connecting your repository)
- Docker installed locally (for building and testing the image)

## Overview

Render's Blueprint feature allows you to define your entire application infrastructure as code using a `render.yaml` file. For aisopod, this includes:

- A Docker-based web service
- Persistent disk storage (1GB)
- Environment variable configuration
- Automatic health checks

## Connection Options

### Option 1: Fork the Repository

If you want to make custom changes to the deployment configuration or the application itself:

1. Navigate to the [aisopod GitHub repository](https://github.com/your-org/aisopod)
2. Click the "Fork" button in the top-right corner
3. Select your GitHub account as the fork destination
4. Clone your fork locally to make changes

### Option 2: Connect the Original Repository

If you want to deploy as-is and receive updates from the main repository:

1. Go to your Render dashboard
2. Click "New +" and select "Blueprint"
3. Connect your preferred Git provider
4. Select the aisopod repository

## Deployment Steps

### Step 1: Create a New Blueprint Instance

1. Log in to your [Render dashboard](https://dashboard.render.com)
2. Click the "New +" button in the top navigation
3. Select "Blueprint Instance" from the dropdown

### Step 2: Connect Your Repository

1. Choose your Git provider (GitHub, GitLab, or Bitbucket)
2. Authorize Render to access your repositories if prompted
3. Select the aisopod repository from the list
4. Click "Continue"

### Step 3: Configure the Blueprint

Render will automatically detect the `render.yaml` file at the repository root. The configuration includes:

- **Service Name**: `aisopod`
- **Runtime**: Docker
- **Dockerfile Path**: `./Dockerfile`
- **Plan**: Starter (shared CPU)
- **Health Check**: `/health` endpoint
- **Disk**: 1GB persistent storage at `/data`

### Step 4: Set Environment Variables

Render will read environment variables from your `render.yaml` file, but you should configure additional secrets through the Render dashboard:

1. After the blueprint is created, navigate to your service
2. Go to the "Environment" tab
3. Add any required secrets:

| Variable | Description | Example |
|----------|-------------|---------|
| `AISOPOD_CONFIG` | Path to configuration file | `/data/config/aisopod.json` |
| `AISOPOD_BIND_ADDRESS` | Server bind address | `0.0.0.0` |
| `AISOPOD_PORT` | Server port | `18789` |

4. Add any API keys or credentials your deployment needs:

| Variable | Description | Example |
|----------|-------------|---------|
| `AISOPOD_PROVIDERS_0_API_KEY` | Provider API key | `sk-...` |
| `AISOPOD_SECRET_KEY` | Application secret key | Random string |
| `AISOPOD_AUTH_PROVIDERS` | Authentication providers | `token,password` |

**Important**: Never commit API keys or sensitive credentials to your repository. Use the Render dashboard to set these as environment variables.

### Step 5: Deploy

1. Click "Create Blueprint Instance"
2. Render will begin building and deploying your service
3. Monitor the build progress in the dashboard

## Monitoring Your Service

### View Logs

1. Navigate to your aisopod service on Render
2. Click the "Logs" tab
3. View real-time logs from your running container
4. Use the search and filter options to find specific entries

### Monitor Health

The `/health` endpoint is automatically monitored:

- Health checks run every 30 seconds
- The endpoint should return HTTP 200 for healthy status
- Unhealthy instances are automatically restarted

To check the health endpoint directly:

```bash
curl https://your-aisopod-instance.onrender.com/health
```

### View Metrics

Render provides basic metrics for your service:

- **CPU Usage**: Current and historical CPU utilization
- **Memory Usage**: Current and historical memory consumption
- **Request Count**: Number of incoming requests
- **Response Time**: Average response times

View these metrics on your service's "Metrics" tab.

### Access the Dashboard

Your aisopod web interface is available at:

```
https://aisopod-<service-id>.onrender.com
```

Or whatever custom domain you've configured.

## Configuration Reference

### Environment Variables (from render.yaml)

| Variable | Value | Description |
|----------|-------|-------------|
| `AISOPOD_CONFIG` | `/data/config/aisopod.json` | Path to the configuration file on the mounted volume |
| `AISOPOD_BIND_ADDRESS` | `0.0.0.0` | Bind to all network interfaces |
| `AISOPOD_PORT` | `18789` | HTTP server port |

### Disk Configuration

- **Name**: `aisopod-data`
- **Mount Path**: `/data`
- **Size**: 1 GB

The disk is automatically created and mounted when the service is deployed. All persistent data is stored here:

- Configuration files
- Session data
- Any other state that needs to survive restarts

## Customizing the Deployment

### Change the Service Plan

1. Navigate to your service on Render
2. Go to the "Settings" tab
3. Click "Change Plan"
4. Select a different plan (Starter, Basic, Standard, or Plus)

### Add Custom Domain

1. Go to the "Settings" tab
2. Scroll to "Custom Domain"
3. Enter your domain name
4. Follow the DNS configuration instructions

### Configure Health Check

The default health check is configured for the `/health` endpoint. To customize:

1. Go to the "Settings" tab
2. Scroll to "Health Check"
3. Modify the path or interval as needed

### Enable HTTPS

HTTPS is enabled by default on Render. Your service is accessible via:

- `https://aisopod-<service-id>.onrender.com`

## Troubleshooting

### Service Fails to Start

1. Check the logs for error messages
2. Verify all required environment variables are set
3. Ensure the Dockerfile builds successfully

### Health Check Fails

1. Test the `/health` endpoint manually:
   ```bash
   curl -v https://your-instance.onrender.com/health
   ```
2. Verify the aisopod binary is working:
   ```bash
   # SSH into the instance
   render ssh
   # Test the health endpoint
   curl http://localhost:18789/health
   ```

### Persistent Disk Issues

1. List volumes in the Render dashboard
2. Check disk usage on the instance:
   ```bash
   render ssh
   df -h /data
   ```

### Environment Variables Not Applied

1. Verify variables are set in the Render dashboard
2. Restart the service after changing environment variables
3. Check for typos in variable names

## Cost Considerations

- **Starter Plan**: Free tier available with limitations
- **Paid Plans**: Based on resource usage and plan tier
- **Disk Storage**: ~$0.15/GB/month

Review the [Render pricing page](https://render.com/pricing) for current rates.

## Next Steps

- Review the [Configuration User Guide](../configuration-user-guide.md) for setting up aisopod
- Check the [Agents, Channels, Skills Guide](../agents-channels-skills-guide.md) for configuring integrations
- Set up custom domains and SSL certificates
- Configure backup and monitoring alerts

## See Also

- [Fly.io Deployment Guide](./flyio.md) - Alternative deployment option
- [Docker Deployment](../../README.md#docker) - Running locally with Docker
- [Configuration Guide](../configuration-user-guide.md) - Configuration reference
