# Docker Deployment

Run sigc in containers for reproducible, scalable deployments.

## Quick Start

### Pull Image

```bash
docker pull ghcr.io/skelf-Research/sigc:latest
```

### Run Backtest

```bash
docker run -v $(pwd):/data ghcr.io/skelf-Research/sigc:latest \
  run /data/strategy.sig
```

### Run Daemon

```bash
docker run -d \
  --name sigc-daemon \
  -v $(pwd)/config:/etc/sigc \
  -v $(pwd)/data:/data \
  -p 5555:5555 \
  -p 9090:9090 \
  ghcr.io/skelf-Research/sigc:latest \
  daemon --config /etc/sigc/sigc.yaml
```

## Dockerfile

### Production Dockerfile

```dockerfile
# Use official sigc image as base
FROM ghcr.io/skelf-Research/sigc:latest

# Set working directory
WORKDIR /app

# Copy strategy files
COPY strategies/ /app/strategies/

# Copy configuration
COPY config/sigc.yaml /etc/sigc/sigc.yaml

# Expose ports
EXPOSE 5555 9090 8080

# Run daemon
CMD ["daemon", "--config", "/etc/sigc/sigc.yaml"]
```

### Build Custom Image

```bash
docker build -t my-sigc:latest .
```

## Docker Compose

### Basic Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  sigc:
    image: ghcr.io/skelf-Research/sigc:latest
    container_name: sigc-daemon
    restart: unless-stopped
    ports:
      - "5555:5555"   # RPC
      - "9090:9090"   # Prometheus
      - "8080:8080"   # Health
    volumes:
      - ./config:/etc/sigc:ro
      - ./strategies:/app/strategies:ro
      - ./data:/data
      - sigc-cache:/var/cache/sigc
      - sigc-logs:/var/log/sigc
    environment:
      - ALPACA_API_KEY=${ALPACA_API_KEY}
      - ALPACA_API_SECRET=${ALPACA_API_SECRET}
      - SIGC_RPC_TOKEN=${SIGC_RPC_TOKEN}
    command: daemon --config /etc/sigc/sigc.yaml

volumes:
  sigc-cache:
  sigc-logs:
```

### With Monitoring Stack

```yaml
# docker-compose.yml
version: '3.8'

services:
  sigc:
    image: ghcr.io/skelf-Research/sigc:latest
    container_name: sigc-daemon
    restart: unless-stopped
    ports:
      - "5555:5555"
      - "9090:9090"
      - "8080:8080"
    volumes:
      - ./config:/etc/sigc:ro
      - ./strategies:/app/strategies:ro
      - ./data:/data
      - sigc-cache:/var/cache/sigc
      - sigc-logs:/var/log/sigc
    environment:
      - ALPACA_API_KEY=${ALPACA_API_KEY}
      - ALPACA_API_SECRET=${ALPACA_API_SECRET}
    command: daemon --config /etc/sigc/sigc.yaml
    networks:
      - sigc-network

  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=90d'
    networks:
      - sigc-network

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    networks:
      - sigc-network

  alertmanager:
    image: prom/alertmanager:latest
    container_name: alertmanager
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml:ro
    networks:
      - sigc-network

volumes:
  sigc-cache:
  sigc-logs:
  prometheus-data:
  grafana-data:

networks:
  sigc-network:
```

## Kubernetes

### Deployment

```yaml
# sigc-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sigc
  labels:
    app: sigc
spec:
  replicas: 1
  selector:
    matchLabels:
      app: sigc
  template:
    metadata:
      labels:
        app: sigc
    spec:
      containers:
        - name: sigc
          image: ghcr.io/skelf-Research/sigc:latest
          args: ["daemon", "--config", "/etc/sigc/sigc.yaml"]
          ports:
            - containerPort: 5555
              name: rpc
            - containerPort: 9090
              name: metrics
            - containerPort: 8080
              name: health
          env:
            - name: ALPACA_API_KEY
              valueFrom:
                secretKeyRef:
                  name: sigc-secrets
                  key: alpaca-api-key
            - name: ALPACA_API_SECRET
              valueFrom:
                secretKeyRef:
                  name: sigc-secrets
                  key: alpaca-api-secret
          volumeMounts:
            - name: config
              mountPath: /etc/sigc
              readOnly: true
            - name: strategies
              mountPath: /app/strategies
              readOnly: true
            - name: cache
              mountPath: /var/cache/sigc
          resources:
            requests:
              memory: "1Gi"
              cpu: "500m"
            limits:
              memory: "4Gi"
              cpu: "2000m"
          livenessProbe:
            httpGet:
              path: /live
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
      volumes:
        - name: config
          configMap:
            name: sigc-config
        - name: strategies
          configMap:
            name: sigc-strategies
        - name: cache
          persistentVolumeClaim:
            claimName: sigc-cache-pvc
```

### Service

```yaml
# sigc-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: sigc
spec:
  selector:
    app: sigc
  ports:
    - name: rpc
      port: 5555
      targetPort: 5555
    - name: metrics
      port: 9090
      targetPort: 9090
    - name: health
      port: 8080
      targetPort: 8080
```

### Secrets

```yaml
# sigc-secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: sigc-secrets
type: Opaque
stringData:
  alpaca-api-key: "your-api-key"
  alpaca-api-secret: "your-api-secret"
```

### ConfigMap

```yaml
# sigc-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: sigc-config
data:
  sigc.yaml: |
    mode: production
    data:
      source: s3://bucket/prices/
    schedule:
      jobs:
        - name: rebalance
          cron: "0 9 * * 1-5"
          strategy: momentum
```

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `ALPACA_API_KEY` | Alpaca API key |
| `ALPACA_API_SECRET` | Alpaca API secret |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `SIGC_CONFIG` | Config file path | `/etc/sigc/sigc.yaml` |
| `SIGC_LOG_LEVEL` | Log level | `info` |
| `SIGC_RPC_TOKEN` | RPC auth token | - |
| `AWS_ACCESS_KEY_ID` | AWS access key | - |
| `AWS_SECRET_ACCESS_KEY` | AWS secret | - |

## Volume Mounts

| Path | Purpose | Mode |
|------|---------|------|
| `/etc/sigc` | Configuration | Read-only |
| `/app/strategies` | Strategy files | Read-only |
| `/data` | Market data | Read/Write |
| `/var/cache/sigc` | Cache | Read/Write |
| `/var/log/sigc` | Logs | Read/Write |

## Health Checks

### Docker

```yaml
services:
  sigc:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
```

### Kubernetes

```yaml
livenessProbe:
  httpGet:
    path: /live
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

## Resource Limits

### Docker

```yaml
services:
  sigc:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '0.5'
          memory: 1G
```

### Kubernetes

```yaml
resources:
  requests:
    memory: "1Gi"
    cpu: "500m"
  limits:
    memory: "4Gi"
    cpu: "2000m"
```

## Logging

### Docker

```yaml
services:
  sigc:
    logging:
      driver: json-file
      options:
        max-size: "100m"
        max-file: "10"
```

### View Logs

```bash
docker logs -f sigc-daemon
```

## Networking

### Docker Network

```bash
# Create network
docker network create sigc-network

# Run with network
docker run --network sigc-network ...
```

### Expose Specific Ports

```bash
# Only expose health check externally
docker run \
  -p 8080:8080 \           # Health (external)
  --expose 5555 \          # RPC (internal only)
  --expose 9090 \          # Metrics (internal only)
  ...
```

## Security

### Run as Non-Root

```dockerfile
# In Dockerfile
RUN useradd -r -u 1000 sigc
USER sigc
```

### Read-Only Root Filesystem

```yaml
services:
  sigc:
    read_only: true
    tmpfs:
      - /tmp
    volumes:
      - sigc-cache:/var/cache/sigc
      - sigc-logs:/var/log/sigc
```

### Drop Capabilities

```yaml
services:
  sigc:
    cap_drop:
      - ALL
    security_opt:
      - no-new-privileges:true
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs sigc-daemon

# Run interactively
docker run -it --entrypoint /bin/sh ghcr.io/skelf-Research/sigc:latest
```

### Health Check Failing

```bash
# Check health endpoint manually
docker exec sigc-daemon curl http://localhost:8080/health
```

### Permission Issues

```bash
# Check user/permissions
docker exec sigc-daemon id
docker exec sigc-daemon ls -la /data
```

## Best Practices

### 1. Use Specific Tags

```yaml
# Good
image: ghcr.io/skelf-Research/sigc:0.10.0

# Bad
image: ghcr.io/skelf-Research/sigc:latest
```

### 2. Mount Secrets Securely

Use Docker secrets or Kubernetes secrets, not environment variables in compose files.

### 3. Set Resource Limits

Always set memory limits to prevent OOM.

### 4. Use Health Checks

Enable health checks for automatic recovery.

### 5. Persist Important Data

Use volumes for cache and logs.

## Next Steps

- [Daemon Mode](daemon-mode.md) - Daemon configuration
- [Monitoring](monitoring.md) - Prometheus setup
- [Configuration](configuration.md) - Full config reference
