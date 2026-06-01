# Docker Deployment Guide

This guide covers containerizing and deploying Fund-My-Cause using Docker and Docker Compose.

## Quick Start

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+

### Run with Docker Compose

```bash
# 1. Copy environment template
cp apps/interface/.env.example apps/interface/.env.local

# 2. Edit with your configuration
nano apps/interface/.env.local

# 3. Build and start services
docker compose up --build
```

The application will be available at `http://localhost:3000`.

## Architecture

### Multi-Stage Build

The Dockerfile uses a two-stage build to minimize image size:

1. **Builder Stage**: Installs dependencies and compiles Next.js with `output: 'standalone'`
2. **Runner Stage**: Contains only the compiled application (~150-250 MB)

### Health Checks

Both services include health checks:

- **Frontend**: HTTP GET to `/` with 30s interval, 10s timeout, 3 retries, 40s start period
- **Stellar**: HTTP GET to `/` with 10s interval, 5s timeout, 10 retries, 30s start period

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `NEXT_PUBLIC_CONTRACT_ID` | Crowdfund contract address | `CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4` |
| `NEXT_PUBLIC_RPC_URL` | Soroban RPC endpoint | `https://soroban-testnet.stellar.org` |
| `NEXT_PUBLIC_NETWORK_PASSPHRASE` | Stellar network passphrase | `Test SDF Network ; September 2015` |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `NEXT_PUBLIC_HORIZON_URL` | Horizon REST API endpoint | - |
| `NEXT_PUBLIC_PINATA_API_KEY` | Pinata IPFS API key | - |
| `NODE_ENV` | Node environment | `production` |

## Docker Compose Services

### `stellar` (Stellar Quickstart)

Local Stellar network with Soroban RPC enabled.

```bash
# Access Horizon API
curl http://localhost:8000/

# Access Soroban RPC
curl -X POST http://localhost:8000/soroban/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getNetwork"}'
```

### `interface` (Production Frontend)

Production-ready Next.js application.

```bash
# View logs
docker compose logs interface

# Restart service
docker compose restart interface

# Execute command in container
docker compose exec interface sh
```

### `interface-dev` (Development Frontend)

Development server with hot reload (optional profile).

```bash
# Start with dev profile
docker compose --profile dev up

# Access at http://localhost:3001
```

## Building the Image Manually

```bash
# Build from repository root
docker build -f apps/interface/Dockerfile -t fund-my-cause:latest .

# Run container
docker run -p 3000:3000 \
  -e NEXT_PUBLIC_CONTRACT_ID=<contract-id> \
  -e NEXT_PUBLIC_RPC_URL=<rpc-url> \
  -e NEXT_PUBLIC_NETWORK_PASSPHRASE="Test SDF Network ; September 2015" \
  fund-my-cause:latest
```

## Production Deployment

### Environment Setup

```bash
# Create production environment file
cat > apps/interface/.env.production.local << EOF
NEXT_PUBLIC_CONTRACT_ID=<mainnet-contract-id>
NEXT_PUBLIC_RPC_URL=https://soroban-mainnet.stellar.org
NEXT_PUBLIC_NETWORK_PASSPHRASE=Public Global Stellar Network ; September 2015
NEXT_PUBLIC_HORIZON_URL=https://horizon.stellar.org
EOF
```

### Docker Compose Production

```bash
# Start services in detached mode
docker compose up -d

# View logs
docker compose logs -f interface

# Stop services
docker compose down

# Stop and remove volumes
docker compose down -v
```

### Kubernetes Deployment

For Kubernetes, use the Docker image with appropriate manifests:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fund-my-cause
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fund-my-cause
  template:
    metadata:
      labels:
        app: fund-my-cause
    spec:
      containers:
      - name: interface
        image: fund-my-cause:latest
        ports:
        - containerPort: 3000
        env:
        - name: NEXT_PUBLIC_CONTRACT_ID
          valueFrom:
            configMapKeyRef:
              name: fund-my-cause-config
              key: contract-id
        - name: NEXT_PUBLIC_RPC_URL
          valueFrom:
            configMapKeyRef:
              name: fund-my-cause-config
              key: rpc-url
        - name: NEXT_PUBLIC_NETWORK_PASSPHRASE
          valueFrom:
            configMapKeyRef:
              name: fund-my-cause-config
              key: network-passphrase
        livenessProbe:
          httpGet:
            path: /
            port: 3000
          initialDelaySeconds: 40
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /
            port: 3000
          initialDelaySeconds: 20
          periodSeconds: 10
```

## Troubleshooting

### Container won't start

```bash
# Check logs
docker compose logs interface

# Verify environment variables
docker compose config

# Rebuild image
docker compose build --no-cache
```

### Health check failing

```bash
# Test endpoint manually
docker compose exec interface curl http://localhost:3000/

# Check container status
docker compose ps

# Increase health check timeouts in docker-compose.yml
```

### Port already in use

```bash
# Find process using port 3000
lsof -i :3000

# Use different port
docker compose -f docker-compose.yml -p fund-my-cause-alt up
```

### Out of disk space

```bash
# Clean up unused images and containers
docker system prune -a

# Remove specific image
docker rmi fund-my-cause:latest
```

## Performance Optimization

### Image Size

Current image size: ~150-250 MB

To further reduce:

```dockerfile
# Use distroless base image
FROM gcr.io/distroless/nodejs20-debian11 AS runner
```

### Build Caching

```bash
# Build with BuildKit for better caching
DOCKER_BUILDKIT=1 docker build -f apps/interface/Dockerfile -t fund-my-cause .
```

### Resource Limits

```yaml
# In docker-compose.yml
services:
  interface:
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

## Security Best Practices

1. **Use specific image tags** (not `latest`)
2. **Scan images for vulnerabilities**: `docker scan fund-my-cause:latest`
3. **Run as non-root user** (Node.js Alpine does this by default)
4. **Use secrets management** for sensitive environment variables
5. **Enable Docker Content Trust**: `export DOCKER_CONTENT_TRUST=1`
6. **Keep base images updated**: `docker pull node:20-alpine`

## Monitoring

### Container Metrics

```bash
# View resource usage
docker stats

# View container logs with timestamps
docker compose logs --timestamps interface
```

### Health Status

```bash
# Check service health
docker compose ps

# Manual health check
docker compose exec interface curl -f http://localhost:3000/ || exit 1
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Build and push Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    file: ./apps/interface/Dockerfile
    push: true
    tags: myregistry/fund-my-cause:${{ github.sha }}
```

### GitLab CI

```yaml
docker_build:
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -f apps/interface/Dockerfile -t fund-my-cause:$CI_COMMIT_SHA .
    - docker push fund-my-cause:$CI_COMMIT_SHA
```
