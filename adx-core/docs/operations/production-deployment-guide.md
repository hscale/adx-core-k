# ADX Core Production Deployment Guide

## Overview
This guide covers the complete production deployment and operational procedures for ADX Core's temporal-first microservices architecture.

## Architecture Summary
- **Backend**: 6 Rust microservices with dual-mode operation (HTTP + Temporal workers)
- **BFF Layer**: 5 optional optimization services (Node.js/TypeScript and Rust)
- **Frontend**: 6 micro-frontends using Module Federation
- **Infrastructure**: Temporal, PostgreSQL, Redis, monitoring stack
- **Deployment**: Kubernetes with independent scaling and Docker Compose fallback

## Pre-Deployment Requirements

### Infrastructure Prerequisites
- Kubernetes cluster (v1.24+) or Docker Swarm
- PostgreSQL 15+ with SSL enabled
- Redis 7+ with authentication
- Temporal Server 1.22+ with UI
- Load balancer (Nginx/HAProxy)
- SSL certificates
- Monitoring stack (Prometheus, Grafana, Loki)

### Security Requirements
- All secrets stored in Kubernetes secrets or environment variables
- Database connections encrypted (SSL/TLS)
- Inter-service communication secured
- Container images scanned for vulnerabilities
- Network policies configured
- RBAC implemented

### Environment Variables
```bash
# Database
POSTGRES_HOST=postgresql
POSTGRES_PORT=5432
POSTGRES_DB=adx_core
POSTGRES_USER=adx_core_user
POSTGRES_PASSWORD=<secure-password>

# Redis
REDIS_URL=redis://:password@redis:6379

# Temporal
TEMPORAL_SERVER_URL=temporal:7233

# Security
JWT_SECRET=<32-character-secret>
BCRYPT_COST=12

# File Storage
S3_BUCKET=adx-core-files-prod
S3_REGION=us-west-2
AWS_ACCESS_KEY_ID=<access-key>
AWS_SECRET_ACCESS_KEY=<secret-key>

# Monitoring
GRAFANA_PASSWORD=<secure-password>
SLACK_WEBHOOK_URL=<webhook-url>
PAGERDUTY_ROUTING_KEY=<routing-key>
```

## Deployment Procedures

### 1. Infrastructure Deployment

#### Kubernetes Deployment
```bash
# Create namespace
kubectl apply -f adx-core/infrastructure/kubernetes/namespace.yaml

# Deploy secrets
kubectl create secret generic database-secret \
  --from-literal=url="postgresql://user:pass@host:5432/db" \
  -n adx-core

kubectl create secret generic redis-secret \
  --from-literal=url="redis://:pass@host:6379" \
  -n adx-core

kubectl create secret generic jwt-secret \
  --from-literal=secret="your-jwt-secret" \
  -n adx-core

# Deploy services
kubectl apply -f adx-core/infrastructure/kubernetes/microservices-deployment.yaml

# Verify deployment
kubectl get pods -n adx-core
kubectl get services -n adx-core
```

#### Docker Compose Deployment
```bash
# Set environment variables
export VERSION=latest
export POSTGRES_PASSWORD=<secure-password>
export REDIS_PASSWORD=<secure-password>
export JWT_SECRET=<secure-secret>

# Deploy infrastructure
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d

# Verify services
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml ps
```

### 2. Database Setup

#### Initial Setup
```bash
# Run migrations
cd adx-core
sqlx migrate run --database-url "$DATABASE_URL"

# Seed initial data
psql "$DATABASE_URL" -f infrastructure/docker/init-db.sql

# Verify setup
psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';"
```

#### Backup Configuration
```bash
# Set up automated backups
crontab -e
# Add: 0 2 * * * /path/to/adx-core/scripts/backup/backup-database.sh

# Test backup
./adx-core/scripts/backup/backup-database.sh

# Test restore
./adx-core/scripts/backup/restore-database.sh -f backup-file.sql.enc.gz
```

### 3. Service Deployment

#### Backend Services
```bash
# Deploy in order (dependencies first)
services=("api-gateway" "auth-service" "user-service" "file-service" "workflow-service" "tenant-service")

for service in "${services[@]}"; do
    echo "Deploying $service..."
    kubectl set image deployment/$service $service=adx-core/$service:$VERSION -n adx-core
    kubectl rollout status deployment/$service -n adx-core
    
    # Health check
    kubectl get pods -l app=$service -n adx-core
done
```

#### BFF Services
```bash
# Deploy BFF services
bff_services=("auth-bff" "tenant-bff" "file-bff" "user-bff" "workflow-bff")

for service in "${bff_services[@]}"; do
    echo "Deploying $service..."
    kubectl set image deployment/$service $service=adx-core/$service:$VERSION -n adx-core
    kubectl rollout status deployment/$service -n adx-core
done
```

#### Frontend Applications
```bash
# Build and deploy micro-frontends
npm run build:all

# Deploy to CDN
aws s3 sync apps/shell/dist/ s3://adx-core-frontend/shell/
aws s3 sync apps/auth/dist/ s3://adx-core-frontend/auth/
aws s3 sync apps/tenant/dist/ s3://adx-core-frontend/tenant/
aws s3 sync apps/file/dist/ s3://adx-core-frontend/file/
aws s3 sync apps/user/dist/ s3://adx-core-frontend/user/
aws s3 sync apps/workflow/dist/ s3://adx-core-frontend/workflow/

# Invalidate CDN cache
aws cloudfront create-invalidation --distribution-id $CLOUDFRONT_DISTRIBUTION_ID --paths "/*"
```

### 4. Monitoring Setup

#### Prometheus Configuration
```bash
# Deploy monitoring stack
kubectl apply -f adx-core/infrastructure/monitoring/prometheus-deployment.yaml

# Verify metrics collection
curl http://prometheus:9090/api/v1/targets
```

#### Grafana Dashboards
```bash
# Import dashboards
curl -X POST http://admin:$GRAFANA_PASSWORD@grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @adx-core/infrastructure/monitoring/grafana/dashboards/system-overview.json

# Verify dashboard access
curl http://admin:$GRAFANA_PASSWORD@grafana:3000/api/dashboards/home
```

#### Log Aggregation
```bash
# Verify Loki is collecting logs
curl http://loki:3100/ready

# Test log query
curl -G -s "http://loki:3100/loki/api/v1/query" \
  --data-urlencode 'query={job="api-gateway"}' \
  --data-urlencode 'limit=10'
```

### 5. Post-Deployment Verification

#### Health Checks
```bash
# Run comprehensive health checks
./adx-core/scripts/deployment-health-check.sh

# Check all service endpoints
services=("api-gateway:8080" "auth-service:8081" "user-service:8082" "file-service:8083" "workflow-service:8084" "tenant-service:8085")

for service in "${services[@]}"; do
    echo "Checking $service..."
    curl -f "http://localhost:${service#*:}/health" || echo "$service health check failed"
done
```

#### Functional Tests
```bash
# Run smoke tests
npm run test:smoke

# Test critical workflows
curl -X POST http://localhost:8080/api/v1/workflows/health-check \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -d '{"test": true}'

# Verify Temporal workflows
curl http://temporal:8088/api/v1/namespaces/default/workflows
```

#### Performance Verification
```bash
# Run load tests
npm run test:load

# Check response times
for i in {1..10}; do
    curl -w "@curl-format.txt" -o /dev/null -s "http://localhost:8080/api/v1/health"
done
```

## Scaling Procedures

### Horizontal Pod Autoscaling (HPA)
```bash
# Check HPA status
kubectl get hpa -n adx-core

# Manual scaling
kubectl scale deployment api-gateway --replicas=5 -n adx-core

# Update HPA thresholds
kubectl patch hpa api-gateway-hpa -n adx-core -p '{"spec":{"maxReplicas":15}}'
```

### Vertical Scaling
```bash
# Update resource limits
kubectl patch deployment api-gateway -n adx-core -p '{
  "spec": {
    "template": {
      "spec": {
        "containers": [{
          "name": "api-gateway",
          "resources": {
            "limits": {"memory": "2Gi", "cpu": "2000m"},
            "requests": {"memory": "512Mi", "cpu": "500m"}
          }
        }]
      }
    }
  }
}'
```

### Database Scaling
```bash
# Scale PostgreSQL (read replicas)
kubectl apply -f adx-core/infrastructure/kubernetes/postgres-replica.yaml

# Update connection strings for read operations
kubectl set env deployment/user-service POSTGRES_READ_URL="postgresql://readonly@postgres-replica:5432/adx_core" -n adx-core
```

## Disaster Recovery

### Backup Procedures
```bash
# Automated daily backups
0 2 * * * /path/to/adx-core/scripts/backup/backup-database.sh

# Manual backup
./adx-core/scripts/backup/backup-database.sh

# Verify backup integrity
./adx-core/scripts/backup/verify-backup.sh backup-file.sql.enc.gz
```

### Recovery Procedures
```bash
# Database recovery
./adx-core/scripts/backup/restore-database.sh -s backup-file.sql.enc.gz

# Service recovery
kubectl rollout restart deployment/api-gateway -n adx-core

# Full system recovery
kubectl apply -f adx-core/infrastructure/kubernetes/microservices-deployment.yaml
```

### Failover Procedures
```bash
# Switch to backup region
kubectl config use-context backup-cluster

# Update DNS to point to backup
aws route53 change-resource-record-sets --hosted-zone-id $ZONE_ID --change-batch file://failover-dns.json

# Verify failover
curl -f https://api.adxcore.com/health
```

## Security Hardening

### Container Security
```bash
# Scan images for vulnerabilities
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image adx-core/api-gateway:latest

# Update base images regularly
docker build --no-cache -t adx-core/api-gateway:latest adx-core/services/api-gateway/
```

### Network Security
```bash
# Apply network policies
kubectl apply -f adx-core/infrastructure/kubernetes/network-policies.yaml

# Verify SSL/TLS configuration
openssl s_client -connect api.adxcore.com:443 -servername api.adxcore.com
```

### Access Control
```bash
# Update RBAC policies
kubectl apply -f adx-core/infrastructure/kubernetes/rbac.yaml

# Rotate secrets
kubectl create secret generic jwt-secret-new --from-literal=secret="new-secret" -n adx-core
kubectl patch deployment api-gateway -n adx-core -p '{"spec":{"template":{"spec":{"containers":[{"name":"api-gateway","env":[{"name":"JWT_SECRET","valueFrom":{"secretKeyRef":{"name":"jwt-secret-new","key":"secret"}}}]}]}}}}'
```

## Troubleshooting

### Common Issues

#### Service Won't Start
1. Check logs: `kubectl logs deployment/service-name -n adx-core`
2. Verify environment variables: `kubectl describe deployment/service-name -n adx-core`
3. Check resource limits: `kubectl top pods -n adx-core`
4. Verify dependencies: `kubectl get services -n adx-core`

#### Database Connection Issues
1. Test connectivity: `kubectl exec -it deployment/api-gateway -n adx-core -- nc -zv postgresql 5432`
2. Check credentials: `kubectl get secret database-secret -n adx-core -o yaml`
3. Verify SSL configuration: `kubectl exec -it deployment/postgresql -n adx-core -- psql -c "SHOW ssl;"`

#### High Memory Usage
1. Check memory metrics: `kubectl top pods -n adx-core`
2. Analyze heap dumps: `kubectl exec -it pod-name -n adx-core -- jstack 1`
3. Update resource limits: `kubectl patch deployment service-name -n adx-core -p '{"spec":{"template":{"spec":{"containers":[{"name":"service-name","resources":{"limits":{"memory":"2Gi"}}}]}}}}'`

#### Temporal Workflow Issues
1. Check Temporal UI: `http://temporal-ui:8088`
2. Verify task queues: `kubectl logs deployment/workflow-service -n adx-core | grep "task_queue"`
3. Check workflow definitions: `kubectl exec -it deployment/workflow-service -n adx-core -- ls /app/workflows/`

## Maintenance Procedures

### Rolling Updates
```bash
# Update service image
kubectl set image deployment/api-gateway api-gateway=adx-core/api-gateway:v2.1.0 -n adx-core

# Monitor rollout
kubectl rollout status deployment/api-gateway -n adx-core

# Rollback if needed
kubectl rollout undo deployment/api-gateway -n adx-core
```

### Database Maintenance
```bash
# Run VACUUM and ANALYZE
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -c "VACUUM ANALYZE;"

# Update statistics
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -c "ANALYZE;"

# Check index usage
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -f /scripts/index-analysis.sql
```

### Certificate Renewal
```bash
# Update TLS certificates
kubectl create secret tls adx-core-tls --cert=cert.pem --key=key.pem -n adx-core

# Update ingress
kubectl patch ingress adx-core-ingress -n adx-core -p '{"spec":{"tls":[{"secretName":"adx-core-tls","hosts":["api.adxcore.com"]}]}}'
```

## Performance Optimization

### Database Optimization
```bash
# Analyze slow queries
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -c "SELECT query, mean_time, calls FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"

# Add indexes
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -c "CREATE INDEX CONCURRENTLY idx_users_tenant_id ON users(tenant_id);"
```

### Cache Optimization
```bash
# Monitor Redis performance
kubectl exec -it deployment/redis -n adx-core -- redis-cli info stats

# Optimize cache settings
kubectl exec -it deployment/redis -n adx-core -- redis-cli config set maxmemory-policy allkeys-lru
```

### Application Optimization
```bash
# Profile application performance
kubectl exec -it deployment/api-gateway -n adx-core -- curl http://localhost:8080/debug/pprof/profile

# Update JVM settings (for BFF services)
kubectl patch deployment auth-bff -n adx-core -p '{"spec":{"template":{"spec":{"containers":[{"name":"auth-bff","env":[{"name":"NODE_OPTIONS","value":"--max-old-space-size=1024"}]}]}}}}'
```

## Compliance and Auditing

### Security Audits
```bash
# Run security audit
./adx-core/scripts/security/security-audit.sh

# Generate compliance report
kubectl exec -it deployment/api-gateway -n adx-core -- /app/scripts/compliance-report.sh
```

### Log Retention
```bash
# Configure log retention
kubectl patch configmap loki-config -n adx-core -p '{"data":{"loki.yaml":"retention_period: 2160h"}}'

# Archive old logs
kubectl exec -it deployment/loki -n adx-core -- /app/scripts/archive-logs.sh
```

## Emergency Procedures

### Incident Response
1. **Assess Impact**: Determine scope and severity
2. **Notify Team**: Alert on-call engineers and stakeholders
3. **Isolate Issue**: Identify and isolate affected components
4. **Implement Fix**: Apply immediate remediation
5. **Verify Resolution**: Confirm system stability
6. **Post-Mortem**: Conduct incident review and documentation

### Emergency Contacts
- **On-call Engineer**: +1-XXX-XXX-XXXX
- **DevOps Team**: devops@adxcore.com
- **Security Team**: security@adxcore.com
- **Management**: management@adxcore.com

### Escalation Matrix
- **P0 (Critical)**: Immediate response, page all teams
- **P1 (High)**: 15-minute response, notify management
- **P2 (Medium)**: 1-hour response, standard team notification
- **P3 (Low)**: Next business day, email notification

## Related Documentation
- [Service Deployment Runbook](./runbooks/service-deployment.md)
- [Monitoring Runbook](./runbooks/monitoring.md)
- [Database Operations](./runbooks/database-operations.md)
- [Security Incident Response](./runbooks/security-incident-response.md)
- [Disaster Recovery](./runbooks/disaster-recovery.md)