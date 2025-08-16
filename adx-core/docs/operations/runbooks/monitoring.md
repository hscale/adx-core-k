# Monitoring and Alerting Runbook

## Overview
This runbook covers monitoring, alerting, and observability procedures for ADX Core production environment.

## Monitoring Stack
- **Prometheus**: Metrics collection and alerting
- **Grafana**: Visualization and dashboards
- **Loki**: Log aggregation
- **Promtail**: Log collection
- **Alertmanager**: Alert routing and notification

## Key Dashboards

### System Overview Dashboard
- **URL**: https://grafana.adxcore.com/d/system-overview
- **Metrics**: CPU, Memory, Disk, Network
- **Services**: All microservices status
- **Alerts**: Active alerts summary

### Application Performance Dashboard
- **URL**: https://grafana.adxcore.com/d/app-performance
- **Metrics**: Request rate, latency, error rate
- **Workflows**: Temporal workflow metrics
- **Database**: Query performance

### Business Metrics Dashboard
- **URL**: https://grafana.adxcore.com/d/business-metrics
- **Metrics**: Active users, tenant usage, revenue
- **Workflows**: Business process completion rates
- **Features**: Feature usage statistics

## Alert Categories

### Critical Alerts (Immediate Response Required)

#### Service Down
- **Trigger**: Service unavailable for > 1 minute
- **Impact**: Complete service outage
- **Response**: Immediate investigation and resolution
- **Escalation**: Page on-call engineer

#### Database Down
- **Trigger**: PostgreSQL/Redis unavailable
- **Impact**: All services affected
- **Response**: Database recovery procedures
- **Escalation**: Page database administrator

#### Security Breach
- **Trigger**: Suspicious activity detected
- **Impact**: Potential data compromise
- **Response**: Security incident response
- **Escalation**: Page security team

### Warning Alerts (Response Within 30 Minutes)

#### High Error Rate
- **Trigger**: Error rate > 5% for > 2 minutes
- **Impact**: Degraded user experience
- **Response**: Investigate error patterns
- **Escalation**: Notify development team

#### High Latency
- **Trigger**: 95th percentile > 500ms for > 5 minutes
- **Impact**: Slow user experience
- **Response**: Performance investigation
- **Escalation**: Notify performance team

#### Resource Usage
- **Trigger**: CPU/Memory > 80% for > 5 minutes
- **Impact**: Potential service degradation
- **Response**: Scale resources or optimize
- **Escalation**: Notify infrastructure team

## Alert Response Procedures

### 1. Alert Acknowledgment
```bash
# Acknowledge alert in Alertmanager
curl -X POST http://alertmanager:9093/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '[{"labels":{"alertname":"ServiceDown","instance":"api-gateway:8080"}}]'
```

### 2. Initial Investigation
- Check service logs
- Verify service health endpoints
- Review recent deployments
- Check infrastructure status

### 3. Service Recovery
```bash
# Restart service
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml restart <service-name>

# Check service status
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml ps <service-name>

# Verify health
curl -f http://localhost:<port>/health
```

### 4. Root Cause Analysis
- Analyze logs and metrics
- Identify contributing factors
- Document findings
- Implement preventive measures

## Log Analysis

### Accessing Logs
```bash
# Service logs via Docker
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml logs <service-name>

# Logs via Loki/Grafana
# Navigate to Grafana > Explore > Loki
# Query: {job="<service-name>"}
```

### Common Log Queries

#### Error Analysis
```logql
# All errors in the last hour
{job=~".*"} |= "ERROR" | json | __error__ = ""

# Specific service errors
{job="api-gateway"} |= "ERROR" | json | line_format "{{.timestamp}} {{.level}} {{.message}}"

# Workflow failures
{job="workflow-service"} |= "workflow_failed" | json
```

#### Performance Analysis
```logql
# Slow requests
{job="api-gateway"} | json | duration > 1000

# Database slow queries
{job="postgresql"} |= "slow query"
```

#### Security Analysis
```logql
# Failed authentication attempts
{job="auth-service"} |= "authentication_failed" | json

# Suspicious activity
{job=~".*"} |= "suspicious_activity" | json
```

## Metrics Analysis

### Key Performance Indicators

#### Request Metrics
```promql
# Request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# Latency (95th percentile)
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

#### Resource Metrics
```promql
# CPU usage
rate(container_cpu_usage_seconds_total[5m])

# Memory usage
container_memory_usage_bytes / container_spec_memory_limit_bytes

# Disk usage
(node_filesystem_size_bytes - node_filesystem_avail_bytes) / node_filesystem_size_bytes
```

#### Business Metrics
```promql
# Active users
active_users_total

# Workflow completion rate
rate(temporal_workflow_completed_total[5m]) / rate(temporal_workflow_started_total[5m])

# Tenant usage
tenant_resource_usage_total
```

## Capacity Planning

### Resource Monitoring
- Monitor CPU, memory, disk, and network usage
- Track growth trends
- Plan for peak usage periods
- Set up predictive alerts

### Scaling Triggers
- CPU usage > 70% sustained
- Memory usage > 80% sustained
- Disk usage > 85%
- Network bandwidth > 80%

### Scaling Procedures
```bash
# Scale service replicas
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml up -d --scale api-gateway=5

# Add new nodes (if using orchestration)
# Follow infrastructure scaling procedures
```

## Maintenance Procedures

### Scheduled Maintenance
1. **Notification**: Notify users 24 hours in advance
2. **Preparation**: Prepare maintenance scripts
3. **Backup**: Create system backups
4. **Execution**: Perform maintenance during low-traffic hours
5. **Verification**: Verify system functionality
6. **Notification**: Confirm maintenance completion

### Emergency Maintenance
1. **Assessment**: Evaluate urgency and impact
2. **Notification**: Immediate notification to stakeholders
3. **Execution**: Perform emergency procedures
4. **Monitoring**: Continuous monitoring during maintenance
5. **Recovery**: Verify system recovery
6. **Post-mortem**: Conduct incident review

## Troubleshooting Guide

### High CPU Usage
1. Identify processes consuming CPU
2. Check for infinite loops or inefficient algorithms
3. Review recent code changes
4. Scale horizontally if needed
5. Optimize code if necessary

### High Memory Usage
1. Check for memory leaks
2. Analyze heap dumps
3. Review caching strategies
4. Increase memory limits if appropriate
5. Optimize data structures

### Database Performance Issues
1. Check slow query logs
2. Analyze query execution plans
3. Review index usage
4. Check connection pool settings
5. Consider read replicas

### Network Issues
1. Check network connectivity
2. Analyze network latency
3. Review load balancer configuration
4. Check DNS resolution
5. Verify firewall rules

## Monitoring Best Practices

### Alert Fatigue Prevention
- Set appropriate thresholds
- Use alert grouping
- Implement alert suppression during maintenance
- Regular alert review and tuning

### Dashboard Design
- Focus on key metrics
- Use consistent color schemes
- Include context and annotations
- Regular dashboard reviews

### Log Management
- Structured logging
- Appropriate log levels
- Log retention policies
- Sensitive data protection

## Emergency Contacts
- On-call Engineer: +1-XXX-XXX-XXXX
- DevOps Team: devops@adxcore.com
- Database Administrator: dba@adxcore.com
- Security Team: security@adxcore.com

## Related Documentation
- [Service Deployment Runbook](./service-deployment.md)
- [Database Operations](./database-operations.md)
- [Security Incident Response](./security-incident-response.md)
- [Disaster Recovery](./disaster-recovery.md)