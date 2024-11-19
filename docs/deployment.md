# Deployment Guide

## Deployment Options

### Docker Deployment

#### Using Docker Compose
```yaml
version: '3.8'
services:
  magicapi:
    image: magicapi/ai-gateway:latest
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
      - PORT=3000
    volumes:
      - ./config.toml:/app/config.toml
```

#### Manual Docker Commands
```bash
# Build the image
docker build -t magicapi-gateway .

# Run the container
docker run -d \
  -p 3000:3000 \
  -e RUST_LOG=info \
  --name magicapi \
  magicapi-gateway
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: magicapi-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: magicapi-gateway
  template:
    metadata:
      labels:
        app: magicapi-gateway
    spec:
      containers:
      - name: magicapi-gateway
        image: magicapi/ai-gateway:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "info"
```

### Cloud Platform Deployments

#### AWS ECS
```bash
# Using AWS Copilot
copilot init --app magicapi --name gateway --type "Load Balanced Web Service"
copilot deploy
```

#### Google Cloud Run
```bash
# Deploy to Cloud Run
gcloud run deploy magicapi-gateway \
  --image gcr.io/your-project/magicapi-gateway \
  --platform managed \
  --port 3000
```

#### Azure Container Apps
```bash
# Deploy to Azure
az containerapp up \
  --name magicapi-gateway \
  --resource-group your-rg \
  --source .
```

## Production Considerations

### Security Checklist
- [ ] Enable TLS/SSL
- [ ] Configure proper CORS settings
- [ ] Set up API key authentication
- [ ] Enable rate limiting
- [ ] Configure proper logging
- [ ] Set up monitoring and alerts

### Performance Optimization
```toml
[server]
max_connections = 1000
worker_threads = 8
tcp_keepalive = true

[performance]
enable_compression = true
compression_level = 6
```

### Monitoring Setup

#### Prometheus & Grafana
```yaml
monitoring:
  prometheus:
    enabled: true
    path: /metrics
  grafana:
    enabled: true
    dashboards:
      - request_metrics
      - error_rates
      - latency_metrics
```

### High Availability Setup

#### Load Balancer Configuration
```nginx
upstream magicapi {
    server magicapi1:3000;
    server magicapi2:3000;
    server magicapi3:3000;
}

server {
    listen 80;
    location / {
        proxy_pass http://magicapi;
    }
}
```

## Scaling Strategies

### Horizontal Scaling
```bash
# Kubernetes scaling
kubectl scale deployment magicapi-gateway --replicas=5

# Docker Swarm scaling
docker service scale magicapi_gateway=5
```

### Resource Allocation
```yaml
resources:
  limits:
    cpu: "2"
    memory: "2Gi"
  requests:
    cpu: "1"
    memory: "1Gi"
```

## Backup and Recovery

### Database Backups
```bash
# Backup script example
#!/bin/bash
DATE=$(date +%Y%m%d)
pg_dump -U postgres magicapi > backup_$DATE.sql
```

### Disaster Recovery Plan
1. Regular backups
2. Multi-region deployment
3. Automated failover
4. Regular recovery testing

## Maintenance

### Update Procedure
```bash
# Rolling update in Kubernetes
kubectl set image deployment/magicapi-gateway \
  magicapi-gateway=magicapi/ai-gateway:new-version

# Docker update
docker-compose pull
docker-compose up -d
```

### Health Checks
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 3
  periodSeconds: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 5
``` 