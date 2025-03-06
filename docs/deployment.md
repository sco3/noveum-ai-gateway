# Deployment Guide

## Deployment Options

### Docker Deployment

#### Using Docker Compose
```yaml
version: '3.8'
services:
  noveum:
    image: noveum/noveum-ai-gateway:latest
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
      - PORT=3000
```

#### Manual Docker Commands
```bash
# Build the image
docker build -t noveum-ai-gateway .

# Run the container
docker run -d \
  -p 3000:3000 \
  -e RUST_LOG=info \
  --name noveum \
  noveum-ai-gateway
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: noveum-ai-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: noveum-ai-gateway
  template:
    metadata:
      labels:
        app: noveum-ai-gateway
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
      containers:
      - name: noveum-ai-gateway
        image: noveum/noveum-ai-gateway:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            cpu: "1"
            memory: "1Gi"
          requests:
            cpu: "500m"
            memory: "512Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
```

+```yaml
+apiVersion: v1
+kind: Service
+metadata:
+  name: noveum-ai-gateway
+spec:
+  selector:
+    app: noveum-ai-gateway
+  ports:
+    - protocol: TCP
+      port: 80
+      targetPort: 3000
+  type: ClusterIP
+```

## Production Considerations

### Security Checklist
- [ ] Enable TLS/SSL
- [ ] Configure proper CORS settings
- [ ] Set up API key authentication
- [ ] Enable rate limiting
- [ ] Configure proper logging
- [ ] Set up monitoring and alerts
- [ ] Configure AWS IAM roles with least privilege
- [ ] Enable AWS CloudTrail for API activity logging
- [ ] Set up AWS VPC endpoints for Bedrock
- [ ] Configure AWS KMS for encryption
- [ ] Implement AWS WAF rules


## Scaling Strategies

### Horizontal Scaling
```bash
# Kubernetes scaling
kubectl scale deployment noveum-ai-gateway --replicas=5

# Docker Swarm scaling
docker service scale noveum_gateway=5
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
kubectl set image deployment/noveum-ai-gateway \
  noveum-ai-gateway=noveum/noveum-ai-gateway:new-version

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