# ADX CORE Deployment Guide

## Overview

ADX CORE is built with a microservices architecture that supports multiple deployment strategies. This guide covers deployment options, infrastructure requirements, and operational procedures.

## Architecture Overview

### Microservices Components
- **Backend Services**: 6 Rust services with dual-mode operation (HTTP + Temporal worker)
- **Frontend Services**: 6 React micro-frontends with Module Federation
- **BFF Services**: 5 optional optimization services (Node.js/TypeScript and Rust)
- **Infrastructure**: Temporal, PostgreSQL, Redis, monitoring stack

### Deployment Patterns
- **Development**: docker-compose with hot-reload
- **Staging**: Kubernetes with GitOps
- **Production**: Kubernetes with blue-green deployment
- **Enterprise**: On-premise or hybrid cloud

## Quick Start

### Prerequisites
- Docker and docker-compose
- Kubernetes cluster (for production)
- PostgreSQL 14+
- Redis 6+
- Node.js 18+ and Rust 1.70+

### Development Deployment
```bash
# Clone repository
git clone https://github.com/adxcore/adx-core.git
cd adx-core

# Start development environment
./scripts/dev-start.sh

# Verify services
curl http://localhost:8080/health
```

### Production Deployment
```bash
# Deploy to Kubernetes
kubectl apply -f infrastructure/kubernetes/

# Verify deployment
kubectl get pods -n adx-core
kubectl get services -n adx-core
```

## Deployment Strategies

### 1. Development Environment
**Use Case**: Local development and testing  
**Infrastructure**: docker-compose  
**Services**: All services with hot-reload  

```bash
# Start development stack
docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d

# Services available at:
# - API Gateway: http://localhost:8080
# - Shell App: http://localhost:3000
# - Temporal UI: http://localhost:8088
# - PostgreSQL: localhost:5432
# - Redis: localhost:6379
```

### 2. Staging Environment
**Use Case**: Integration testing and QA  
**Infrastructure**: Kubernetes  
**Services**: Production-like with debug enabled  

```bash
# Deploy staging environment
kubectl apply -f infrastructure/kubernetes/staging/

# Configure staging-specific settings
kubectl create configmap adx-core-config \
  --from-file=config/staging.toml \
  -n adx-core-staging
```

### 3. Production Environment
**Use Case**: Live production workloads  
**Infrastructure**: Kubernetes with HA  
**Services**: Optimized with monitoring  

```bash
# Deploy production environment
kubectl apply -f infrastructure/kubernetes/production/

# Verify deployment health
kubectl get pods -n adx-core-production
kubectl logs -f deployment/api-gateway -n adx-core-production
```

### 4. Enterprise On-Premise
**Use Case**: Enterprise customers with data residency requirements  
**Infrastructure**: Customer-managed Kubernetes or Docker Swarm  
**Services**: White-label configuration  

## Infrastructure Requirements

### Minimum Requirements (Development)
- **CPU**: 4 cores
- **Memory**: 8 GB RAM
- **Storage**: 50 GB SSD
- **Network**: 100 Mbps

### Recommended Requirements (Production)
- **CPU**: 16 cores (distributed across nodes)
- **Memory**: 32 GB RAM (distributed across nodes)
- **Storage**: 500 GB SSD (with backup)
- **Network**: 1 Gbps
- **High Availability**: 3+ nodes

### Kubernetes Cluster Specifications
```yaml
# Minimum cluster configuration
nodes: 3
node_specs:
  cpu: 4 cores
  memory: 16 GB
  storage: 100 GB SSD
  
# Production cluster configuration  
nodes: 6
node_specs:
  cpu: 8 cores
  memory: 32 GB
  storage: 200 GB SSD
  
# Storage classes
storage_classes:
  - name: fast-ssd
    provisioner: kubernetes.io/aws-ebs
    parameters:
      type: gp3
  - name: backup-storage
    provisioner: kubernetes.io/aws-ebs
    parameters:
      type: sc1
```

## Service Configuration

### Backend Services Configuration
```toml
# config/production.toml
[database]
url = "postgresql://user:pass@postgres:5432/adxcore"
max_connections = 20
connection_timeout = 30

[redis]
url = "redis://redis:6379"
pool_size = 10

[temporal]
server_url = "temporal:7233"
namespace = "adx-core-production"
task_queue = "adx-core-tasks"

[auth]
jwt_secret = "${JWT_SECRET}"
jwt_expiry = 3600
refresh_token_expiry = 604800

[file_storage]
provider = "s3"
bucket = "adx-core-files-prod"
region = "us-west-2"

[monitoring]
metrics_enabled = true
tracing_enabled = true
log_level = "info"
```

### Frontend Configuration
```javascript
// apps/shell/src/config/production.js
export const config = {
  apiBaseUrl: 'https://api.adxcore.com',
  bffServices: {
    auth: 'https://auth-bff.adxcore.com',
    tenant: 'https://tenant-bff.adxcore.com',
    file: 'https://file-bff.adxcore.com',
    user: 'https://user-bff.adxcore.com',
    workflow: 'https://workflow-bff.adxcore.com'
  },
  moduleFederation: {
    remotes: {
      auth_app: 'https://auth.adxcore.com/remoteEntry.js',
      tenant_app: 'https://tenant.adxcore.com/remoteEntry.js',
      file_app: 'https://files.adxcore.com/remoteEntry.js',
      user_app: 'https://users.adxcore.com/remoteEntry.js',
      workflow_app: 'https://workflows.adxcore.com/remoteEntry.js',
      module_app: 'https://modules.adxcore.com/remoteEntry.js'
    }
  },
  features: {
    enableAnalytics: true,
    enableErrorReporting: true,
    enablePerformanceMonitoring: true
  }
};
```

## Deployment Procedures

### Automated Deployment (GitOps)
```yaml
# .github/workflows/deploy-production.yml
name: Deploy to Production

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build and Push Images
        run: |
          docker build -t adxcore/api-gateway:${{ github.sha }} .
          docker push adxcore/api-gateway:${{ github.sha }}
      
      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/api-gateway \
            api-gateway=adxcore/api-gateway:${{ github.sha }} \
            -n adx-core-production
          
          kubectl rollout status deployment/api-gateway \
            -n adx-core-production
```

### Manual Deployment Steps
```bash
# 1. Build and tag images
./scripts/build-images.sh production

# 2. Push to registry
./scripts/push-images.sh production

# 3. Update Kubernetes manifests
./scripts/update-manifests.sh production $VERSION

# 4. Apply changes
kubectl apply -f infrastructure/kubernetes/production/

# 5. Verify deployment
./scripts/verify-deployment.sh production

# 6. Run health checks
./scripts/health-check.sh production
```

### Blue-Green Deployment
```bash
# 1. Deploy to green environment
kubectl apply -f infrastructure/kubernetes/production/green/

# 2. Verify green environment
./scripts/verify-deployment.sh green

# 3. Switch traffic to green
kubectl patch service api-gateway \
  -p '{"spec":{"selector":{"version":"green"}}}' \
  -n adx-core-production

# 4. Monitor for issues
./scripts/monitor-deployment.sh green 300

# 5. Clean up blue environment (after verification)
kubectl delete -f infrastructure/kubernetes/production/blue/
```

### Rollback Procedures
```bash
# Quick rollback using Kubernetes
kubectl rollout undo deployment/api-gateway -n adx-core-production

# Rollback to specific version
kubectl rollout undo deployment/api-gateway \
  --to-revision=2 -n adx-core-production

# Verify rollback
kubectl rollout status deployment/api-gateway -n adx-core-production

# Full environment rollback
./scripts/rollback-environment.sh production v2.1.0
```

## Monitoring and Observability

### Metrics Collection
```yaml
# Prometheus configuration
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'adx-core-services'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names: ['adx-core-production']
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
```

### Logging Configuration
```yaml
# Loki configuration for log aggregation
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h
```

### Alerting Rules
```yaml
# Prometheus alerting rules
groups:
  - name: adx-core-alerts
    rules:
      - alert: ServiceDown
        expr: up{job="adx-core-services"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "ADX Core service is down"
          description: "Service {{ $labels.instance }} has been down for more than 1 minute"
      
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"
      
      - alert: WorkflowFailureRate
        expr: rate(workflow_executions_total{status="failed"}[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High workflow failure rate"
          description: "Workflow failure rate is {{ $value }} failures per second"
```

## Security Configuration

### TLS/SSL Setup
```bash
# Generate certificates using cert-manager
kubectl apply -f https://github.com/jetstack/cert-manager/releases/download/v1.8.0/cert-manager.yaml

# Create certificate issuer
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@adxcore.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

### Network Policies
```yaml
# Network policy for service isolation
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: adx-core-network-policy
  namespace: adx-core-production
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: adx-core-production
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: adx-core-production
  - to: []
    ports:
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53
```

## Backup and Recovery

### Database Backup
```bash
# Automated daily backup
#!/bin/bash
# scripts/backup-database.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="adxcore_backup_${DATE}.sql"

# Create backup
pg_dump -h postgres -U adxcore -d adxcore > /backups/${BACKUP_FILE}

# Compress backup
gzip /backups/${BACKUP_FILE}

# Upload to S3
aws s3 cp /backups/${BACKUP_FILE}.gz s3://adx-core-backups/database/

# Clean up local files older than 7 days
find /backups -name "*.gz" -mtime +7 -delete
```

### Application State Backup
```bash
# Backup Temporal workflows and Redis data
#!/bin/bash
# scripts/backup-application-state.sh

DATE=$(date +%Y%m%d_%H%M%S)

# Backup Redis data
redis-cli --rdb /backups/redis_${DATE}.rdb

# Export Temporal workflows
temporal workflow list --namespace adx-core-production \
  --output json > /backups/temporal_workflows_${DATE}.json

# Upload to S3
aws s3 cp /backups/redis_${DATE}.rdb s3://adx-core-backups/redis/
aws s3 cp /backups/temporal_workflows_${DATE}.json s3://adx-core-backups/temporal/
```

### Disaster Recovery
```bash
# Full environment recovery procedure
#!/bin/bash
# scripts/disaster-recovery.sh

BACKUP_DATE=$1

# 1. Restore database
aws s3 cp s3://adx-core-backups/database/adxcore_backup_${BACKUP_DATE}.sql.gz .
gunzip adxcore_backup_${BACKUP_DATE}.sql.gz
psql -h postgres -U adxcore -d adxcore < adxcore_backup_${BACKUP_DATE}.sql

# 2. Restore Redis data
aws s3 cp s3://adx-core-backups/redis/redis_${BACKUP_DATE}.rdb .
redis-cli --rdb redis_${BACKUP_DATE}.rdb

# 3. Redeploy services
kubectl apply -f infrastructure/kubernetes/production/

# 4. Verify recovery
./scripts/verify-deployment.sh production
```

## Troubleshooting

### Common Issues

#### Service Won't Start
```bash
# Check pod status
kubectl get pods -n adx-core-production

# Check pod logs
kubectl logs -f pod/api-gateway-xxx -n adx-core-production

# Check events
kubectl get events -n adx-core-production --sort-by='.lastTimestamp'

# Check resource usage
kubectl top pods -n adx-core-production
```

#### Database Connection Issues
```bash
# Test database connectivity
kubectl exec -it pod/api-gateway-xxx -n adx-core-production -- \
  psql -h postgres -U adxcore -d adxcore -c "SELECT 1"

# Check database logs
kubectl logs -f pod/postgres-xxx -n adx-core-production

# Verify database configuration
kubectl get configmap adx-core-config -o yaml -n adx-core-production
```

#### Temporal Workflow Issues
```bash
# Check Temporal server status
kubectl exec -it pod/temporal-xxx -n adx-core-production -- \
  temporal server status

# List failed workflows
kubectl exec -it pod/temporal-xxx -n adx-core-production -- \
  temporal workflow list --query "ExecutionStatus='Failed'"

# Check workflow history
kubectl exec -it pod/temporal-xxx -n adx-core-production -- \
  temporal workflow show --workflow-id wf_123456
```

### Performance Tuning

#### Database Optimization
```sql
-- Optimize PostgreSQL for production
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
SELECT pg_reload_conf();
```

#### Redis Optimization
```bash
# Redis configuration for production
echo "maxmemory 512mb" >> /etc/redis/redis.conf
echo "maxmemory-policy allkeys-lru" >> /etc/redis/redis.conf
echo "save 900 1" >> /etc/redis/redis.conf
echo "save 300 10" >> /etc/redis/redis.conf
echo "save 60 10000" >> /etc/redis/redis.conf
```

#### Kubernetes Resource Limits
```yaml
# Optimized resource limits
resources:
  requests:
    memory: "256Mi"
    cpu: "250m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

## Support and Maintenance

### Regular Maintenance Tasks
- **Daily**: Check service health, review logs, verify backups
- **Weekly**: Update dependencies, review security alerts, performance analysis
- **Monthly**: Capacity planning, security audit, disaster recovery testing
- **Quarterly**: Major version updates, infrastructure review

### Support Contacts
- **Operations Team**: ops@adxcore.com
- **Security Team**: security@adxcore.com
- **Development Team**: dev@adxcore.com
- **Emergency Hotline**: +1-555-ADX-CORE

### Documentation Updates
This deployment guide is maintained in the ADX CORE repository. For updates or corrections, please submit a pull request or contact the operations team.