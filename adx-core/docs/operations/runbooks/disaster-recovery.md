# Disaster Recovery Runbook

## Overview
This runbook covers disaster recovery procedures for ADX Core production environment, including backup strategies, recovery procedures, and business continuity planning.

## Recovery Time Objectives (RTO) and Recovery Point Objectives (RPO)

### Service Level Objectives
- **RTO**: 4 hours maximum downtime
- **RPO**: 1 hour maximum data loss
- **Availability Target**: 99.9% uptime
- **Data Integrity**: Zero tolerance for data corruption

### Service Priorities
1. **Critical (P0)**: API Gateway, Auth Service, Database
2. **High (P1)**: User Service, Tenant Service, File Service
3. **Medium (P2)**: Workflow Service, BFF Services
4. **Low (P3)**: Monitoring, Logging, Analytics

## Backup Strategy

### Database Backups
- **Frequency**: Every 6 hours
- **Retention**: 30 days local, 90 days remote
- **Encryption**: AES-256 encryption at rest
- **Verification**: Daily integrity checks
- **Storage**: Local + AWS S3 + Geographic replication

### Application Backups
- **Configuration**: Daily snapshots of Kubernetes manifests
- **Secrets**: Encrypted backup of all secrets and certificates
- **Code**: Git repositories with multiple remotes
- **Images**: Container registry with multi-region replication

### Infrastructure Backups
- **Infrastructure as Code**: Terraform state backups
- **Network Configuration**: Daily exports of network policies
- **Monitoring Configuration**: Prometheus/Grafana config backups
- **DNS Records**: Daily exports of DNS configurations

## Disaster Scenarios

### Scenario 1: Single Service Failure

#### Detection
- Service health checks fail
- Increased error rates in monitoring
- User reports of service unavailability

#### Response Procedure
```bash
# 1. Assess impact
kubectl get pods -n adx-core | grep -v Running

# 2. Check service logs
kubectl logs deployment/failed-service -n adx-core --tail=100

# 3. Restart service
kubectl rollout restart deployment/failed-service -n adx-core

# 4. Monitor recovery
kubectl rollout status deployment/failed-service -n adx-core

# 5. Verify functionality
curl -f http://failed-service:port/health
```

#### Escalation
- If restart fails: Scale to zero and back up
- If persistent: Deploy previous known-good version
- If critical service: Activate incident response

### Scenario 2: Database Failure

#### Detection
- Database connection failures
- Data inconsistency reports
- Backup verification failures

#### Response Procedure
```bash
# 1. Assess database status
kubectl exec -it deployment/postgresql -n adx-core -- pg_isready

# 2. Check for corruption
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -c "SELECT pg_database_size('adx_core');"

# 3. If corrupted, initiate recovery
./adx-core/scripts/backup/restore-database.sh -s latest-backup.sql.enc.gz

# 4. Verify data integrity
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -f /scripts/data-integrity-check.sql

# 5. Restart dependent services
kubectl rollout restart deployment/api-gateway -n adx-core
kubectl rollout restart deployment/auth-service -n adx-core
```

#### Data Loss Assessment
```bash
# Check last successful backup
aws s3 ls s3://adx-core-backups/database/ --recursive | tail -1

# Compare with current data
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -c "SELECT MAX(created_at) FROM audit_log;"

# Calculate data loss window
echo "Data loss window: $(date -d 'last backup time' '+%Y-%m-%d %H:%M:%S') to $(date '+%Y-%m-%d %H:%M:%S')"
```

### Scenario 3: Complete Infrastructure Failure

#### Detection
- All services unreachable
- Kubernetes cluster unresponsive
- Infrastructure monitoring alerts

#### Response Procedure
```bash
# 1. Activate disaster recovery site
kubectl config use-context disaster-recovery-cluster

# 2. Deploy infrastructure
terraform apply -var-file="disaster-recovery.tfvars"

# 3. Restore database
./adx-core/scripts/backup/restore-database.sh -s -f latest-backup.sql.enc.gz

# 4. Deploy services
kubectl apply -f adx-core/infrastructure/kubernetes/microservices-deployment.yaml

# 5. Update DNS
aws route53 change-resource-record-sets --hosted-zone-id $ZONE_ID --change-batch file://disaster-recovery-dns.json

# 6. Verify full functionality
./adx-core/scripts/deployment-health-check.sh
```

### Scenario 4: Data Center Outage

#### Detection
- Complete loss of primary region
- Network connectivity issues
- Cloud provider status page alerts

#### Response Procedure
```bash
# 1. Activate secondary region
export AWS_DEFAULT_REGION=us-east-1
kubectl config use-context secondary-region-cluster

# 2. Verify backup availability
aws s3 ls s3://adx-core-backups-secondary/database/

# 3. Deploy full stack
terraform workspace select secondary-region
terraform apply -auto-approve

# 4. Restore data
./adx-core/scripts/backup/restore-database.sh -s latest-cross-region-backup.sql.enc.gz

# 5. Update global DNS
aws route53 change-resource-record-sets --hosted-zone-id $GLOBAL_ZONE_ID --change-batch file://failover-dns.json

# 6. Notify users
curl -X POST "$NOTIFICATION_WEBHOOK" -d '{"message":"Service restored on secondary region"}'
```

## Recovery Procedures

### Database Recovery

#### Point-in-Time Recovery
```bash
# 1. Stop all services
kubectl scale deployment --all --replicas=0 -n adx-core

# 2. Create recovery database
kubectl exec -it deployment/postgresql -n adx-core -- createdb adx_core_recovery

# 3. Restore to specific point in time
./adx-core/scripts/backup/restore-database.sh --point-in-time "2024-01-15 14:30:00" adx_core_recovery

# 4. Verify data integrity
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core_recovery -f /scripts/data-verification.sql

# 5. Switch to recovery database
kubectl set env deployment/api-gateway DATABASE_URL="postgresql://user:pass@postgresql:5432/adx_core_recovery" -n adx-core

# 6. Restart services
kubectl scale deployment --all --replicas=2 -n adx-core
```

#### Cross-Region Recovery
```bash
# 1. Download backup from secondary region
aws s3 cp s3://adx-core-backups-us-east-1/database/latest.sql.enc.gz ./

# 2. Restore database
./adx-core/scripts/backup/restore-database.sh -f latest.sql.enc.gz

# 3. Update replication settings
kubectl exec -it deployment/postgresql -n adx-core -- psql -d adx_core -c "SELECT pg_promote();"

# 4. Reconfigure services
kubectl apply -f adx-core/infrastructure/kubernetes/cross-region-config.yaml
```

### Service Recovery

#### Rolling Back Deployments
```bash
# 1. Check deployment history
kubectl rollout history deployment/api-gateway -n adx-core

# 2. Rollback to previous version
kubectl rollout undo deployment/api-gateway -n adx-core

# 3. Rollback to specific revision
kubectl rollout undo deployment/api-gateway --to-revision=3 -n adx-core

# 4. Verify rollback
kubectl rollout status deployment/api-gateway -n adx-core
```

#### Emergency Service Bypass
```bash
# 1. Deploy minimal service version
kubectl apply -f adx-core/infrastructure/kubernetes/emergency-services.yaml

# 2. Update load balancer to bypass failed services
kubectl patch service api-gateway -n adx-core -p '{"spec":{"selector":{"app":"api-gateway-emergency"}}}'

# 3. Enable maintenance mode
kubectl create configmap maintenance-mode --from-literal=enabled=true -n adx-core
```

### Infrastructure Recovery

#### Kubernetes Cluster Recovery
```bash
# 1. Create new cluster
eksctl create cluster --config-file=adx-core/infrastructure/kubernetes/cluster-config.yaml

# 2. Restore cluster state
kubectl apply -f adx-core/infrastructure/kubernetes/

# 3. Restore persistent volumes
kubectl apply -f adx-core/infrastructure/kubernetes/persistent-volumes.yaml

# 4. Verify cluster health
kubectl get nodes
kubectl get pods --all-namespaces
```

#### Network Recovery
```bash
# 1. Restore network policies
kubectl apply -f adx-core/infrastructure/kubernetes/network-policies.yaml

# 2. Update security groups
aws ec2 authorize-security-group-ingress --group-id sg-12345678 --protocol tcp --port 443 --cidr 0.0.0.0/0

# 3. Verify connectivity
kubectl exec -it deployment/api-gateway -n adx-core -- nc -zv external-service.com 443
```

## Business Continuity

### Communication Plan

#### Internal Communication
1. **Incident Commander**: Coordinates response efforts
2. **Technical Lead**: Manages technical recovery
3. **Communications Lead**: Handles stakeholder updates
4. **Business Lead**: Assesses business impact

#### External Communication
```bash
# Status page update
curl -X POST "https://api.statuspage.io/v1/pages/$PAGE_ID/incidents" \
  -H "Authorization: OAuth $STATUSPAGE_TOKEN" \
  -d '{
    "incident": {
      "name": "Service Disruption",
      "status": "investigating",
      "impact_override": "major"
    }
  }'

# Customer notification
curl -X POST "$CUSTOMER_NOTIFICATION_API" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "We are experiencing technical difficulties and are working to resolve them.",
    "severity": "high",
    "channels": ["email", "sms", "push"]
  }'
```

### Service Degradation

#### Graceful Degradation Modes
```bash
# Enable read-only mode
kubectl create configmap service-mode --from-literal=mode=readonly -n adx-core

# Disable non-critical features
kubectl set env deployment/api-gateway FEATURE_ANALYTICS=false -n adx-core
kubectl set env deployment/api-gateway FEATURE_REPORTING=false -n adx-core

# Reduce service replicas to conserve resources
kubectl scale deployment/workflow-service --replicas=1 -n adx-core
kubectl scale deployment/file-service --replicas=1 -n adx-core
```

#### Load Shedding
```bash
# Enable rate limiting
kubectl patch configmap nginx-config -n adx-core -p '{"data":{"rate-limit":"10r/s"}}'

# Reject non-essential requests
kubectl set env deployment/api-gateway REJECT_NON_ESSENTIAL=true -n adx-core

# Prioritize authenticated users
kubectl set env deployment/api-gateway PRIORITY_AUTH_USERS=true -n adx-core
```

## Testing and Validation

### Disaster Recovery Testing

#### Monthly DR Tests
```bash
# 1. Schedule maintenance window
echo "DR test scheduled for $(date -d 'next saturday 2am')"

# 2. Create test environment
terraform workspace new dr-test-$(date +%Y%m%d)
terraform apply -var-file="dr-test.tfvars"

# 3. Test backup restoration
./adx-core/scripts/backup/restore-database.sh -s test-backup.sql.enc.gz

# 4. Validate functionality
npm run test:dr-validation

# 5. Document results
echo "DR test results: $(date)" >> dr-test-log.txt
```

#### Chaos Engineering
```bash
# Install chaos engineering tools
kubectl apply -f https://raw.githubusercontent.com/chaos-mesh/chaos-mesh/master/manifests/crd.yaml

# Test pod failures
kubectl apply -f adx-core/infrastructure/chaos/pod-failure-test.yaml

# Test network partitions
kubectl apply -f adx-core/infrastructure/chaos/network-partition-test.yaml

# Monitor system behavior
kubectl logs -f deployment/api-gateway -n adx-core
```

### Recovery Validation

#### Data Integrity Checks
```sql
-- Check record counts
SELECT 
    'users' as table_name, COUNT(*) as record_count 
FROM users
UNION ALL
SELECT 
    'tenants' as table_name, COUNT(*) as record_count 
FROM tenants
UNION ALL
SELECT 
    'files' as table_name, COUNT(*) as record_count 
FROM files;

-- Check data consistency
SELECT 
    u.id, u.tenant_id, t.id as tenant_exists
FROM users u
LEFT JOIN tenants t ON u.tenant_id = t.id
WHERE t.id IS NULL;

-- Check recent activity
SELECT 
    MAX(created_at) as last_activity,
    COUNT(*) as recent_records
FROM audit_log
WHERE created_at > NOW() - INTERVAL '1 hour';
```

#### Service Validation
```bash
# Test critical workflows
curl -X POST http://api-gateway:8080/api/v1/workflows/health-check \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -d '{"test_type": "full_system"}'

# Validate user authentication
curl -X POST http://auth-service:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "test123"}'

# Test file operations
curl -X POST http://file-service:8083/api/v1/files/upload \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -F "file=@test-file.txt"
```

## Documentation and Reporting

### Incident Documentation
```bash
# Create incident report
cat > incident-report-$(date +%Y%m%d).md << EOF
# Incident Report - $(date)

## Summary
- **Incident ID**: INC-$(date +%Y%m%d)-001
- **Start Time**: $(date -d '1 hour ago')
- **End Time**: $(date)
- **Duration**: 1 hour
- **Impact**: Service degradation

## Timeline
- $(date -d '1 hour ago'): Issue detected
- $(date -d '45 minutes ago'): Response team activated
- $(date -d '30 minutes ago'): Root cause identified
- $(date -d '15 minutes ago'): Fix implemented
- $(date): Service fully restored

## Root Cause
[Description of root cause]

## Resolution
[Description of resolution steps]

## Lessons Learned
[Key takeaways and improvements]

## Action Items
- [ ] Update monitoring alerts
- [ ] Improve documentation
- [ ] Implement preventive measures
EOF
```

### Recovery Metrics
```bash
# Calculate RTO/RPO metrics
echo "RTO: $(date -d 'incident end' '+%s') - $(date -d 'incident start' '+%s') seconds"
echo "RPO: $(date -d 'last backup' '+%s') - $(date -d 'incident start' '+%s') seconds"

# Generate availability report
kubectl exec -it deployment/prometheus -n adx-core -- promtool query instant 'up{job="api-gateway"}'
```

## Emergency Contacts

### Primary Contacts
- **Incident Commander**: +1-XXX-XXX-XXXX
- **Technical Lead**: +1-XXX-XXX-XXXX
- **Database Administrator**: +1-XXX-XXX-XXXX
- **Security Lead**: +1-XXX-XXX-XXXX

### Vendor Contacts
- **Cloud Provider Support**: +1-XXX-XXX-XXXX
- **Database Vendor**: +1-XXX-XXX-XXXX
- **Monitoring Vendor**: +1-XXX-XXX-XXXX

### Escalation Matrix
- **Level 1**: On-call engineer (immediate)
- **Level 2**: Team lead (15 minutes)
- **Level 3**: Engineering manager (30 minutes)
- **Level 4**: CTO/VP Engineering (1 hour)

## Related Documentation
- [Service Deployment Runbook](./service-deployment.md)
- [Monitoring Runbook](./monitoring.md)
- [Security Incident Response](./security-incident-response.md)
- [Production Deployment Guide](../production-deployment-guide.md)