# Service Deployment Runbook

## Overview
This runbook covers the deployment procedures for ADX Core microservices in production environments.

## Prerequisites
- Access to production environment
- Docker and Docker Compose installed
- Environment variables configured
- Database migrations ready
- Monitoring systems operational

## Deployment Process

### 1. Pre-Deployment Checklist
- [ ] All tests passing in CI/CD pipeline
- [ ] Security scans completed
- [ ] Database migrations tested
- [ ] Rollback plan prepared
- [ ] Monitoring alerts configured
- [ ] Team notified of deployment

### 2. Backend Service Deployment

#### API Gateway Deployment
```bash
# 1. Build and tag image
docker build -t adx-core/api-gateway:${VERSION} adx-core/services/api-gateway/

# 2. Push to registry
docker push adx-core/api-gateway:${VERSION}

# 3. Update environment variables
export VERSION=${VERSION}

# 4. Deploy with rolling update
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d api-gateway

# 5. Verify deployment
curl -f http://localhost:8080/health || echo "Health check failed"

# 6. Check logs
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml logs api-gateway
```

#### Individual Service Deployment
```bash
# Deploy auth-service
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d auth-service

# Deploy user-service
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d user-service

# Deploy file-service
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d file-service

# Deploy workflow-service
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d workflow-service

# Deploy tenant-service
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d tenant-service
```

### 3. BFF Service Deployment
```bash
# Deploy BFF services
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d auth-bff tenant-bff file-bff user-bff workflow-bff
```

### 4. Frontend Deployment

#### Build and Deploy Micro-Frontends
```bash
# Build all micro-frontends
npm run build:all

# Deploy to CDN/Static hosting
aws s3 sync apps/shell/dist/ s3://adx-core-frontend/shell/
aws s3 sync apps/auth/dist/ s3://adx-core-frontend/auth/
aws s3 sync apps/tenant/dist/ s3://adx-core-frontend/tenant/
aws s3 sync apps/file/dist/ s3://adx-core-frontend/file/
aws s3 sync apps/user/dist/ s3://adx-core-frontend/user/
aws s3 sync apps/workflow/dist/ s3://adx-core-frontend/workflow/

# Invalidate CDN cache
aws cloudfront create-invalidation --distribution-id ${CLOUDFRONT_DISTRIBUTION_ID} --paths "/*"
```

### 5. Post-Deployment Verification

#### Health Checks
```bash
# Check all service health endpoints
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
  -d '{"test": true}'
```

#### Monitoring Verification
- Check Grafana dashboards
- Verify Prometheus targets are up
- Confirm log aggregation is working
- Test alerting rules

### 6. Rollback Procedure

#### Automatic Rollback Triggers
- Health check failures for > 5 minutes
- Error rate > 5% for > 2 minutes
- Response time > 2 seconds for > 5 minutes

#### Manual Rollback
```bash
# Rollback to previous version
export PREVIOUS_VERSION=${PREVIOUS_VERSION}

# Update services
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d

# Verify rollback
./scripts/deployment-health-check.sh
```

## Troubleshooting

### Common Issues

#### Service Won't Start
1. Check logs: `docker-compose logs <service-name>`
2. Verify environment variables
3. Check database connectivity
4. Verify Temporal connectivity

#### Database Connection Issues
1. Check PostgreSQL status
2. Verify connection strings
3. Check network connectivity
4. Review database logs

#### High Memory Usage
1. Check container resource limits
2. Review application metrics
3. Analyze memory leaks
4. Scale horizontally if needed

#### Temporal Workflow Issues
1. Check Temporal UI
2. Verify workflow definitions
3. Check activity implementations
4. Review task queue status

## Emergency Contacts
- On-call Engineer: +1-XXX-XXX-XXXX
- DevOps Team: devops@adxcore.com
- Security Team: security@adxcore.com
- Management: management@adxcore.com

## Related Documentation
- [Monitoring Runbook](./monitoring.md)
- [Database Operations](./database-operations.md)
- [Security Incident Response](./security-incident-response.md)
- [Disaster Recovery](./disaster-recovery.md)