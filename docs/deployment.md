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


## Production Considerations

### Security Checklist
- [ ] Enable TLS/SSL
- [ ] Configure proper CORS settings
- [ ] Set up API key authentication
- [ ] Enable rate limiting
- [ ] Configure proper logging
- [ ] Set up monitoring and alerts

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
``` 