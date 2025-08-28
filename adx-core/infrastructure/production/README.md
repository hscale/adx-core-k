# ADX Core Production Deployment Guide

This directory contains all the necessary configuration and scripts for deploying ADX Core to production with comprehensive monitoring, backup, and disaster recovery capabilities.

## Quick Start

1. **Copy and configure environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env with your production values
   ```

2. **Set up SSL certificates:**
   ```bash
   # Place your SSL certificates in nginx/ssl/
   cp your-cert.pem nginx/ssl/cert.pem
   cp your-key.pem nginx/ssl/key.pem
   ```

3. **Deploy to production:**
   ```bash
   sudo ./scripts/deploy.sh deploy
   ```

## Directory Structure

```
production/
├── docker-compose.prod.yml          # Main production services
├── docker-compose.monitoring.yml    # Monitoring stack
├── .env.example                     # Environment variables template
├── nginx/
│   └── nginx.conf                   # Reverse proxy configuration
├── monitoring/
│   ├── prometheus.yml               # Metrics collection
│   ├── alert_rules.yml              # Alert definitions
│   ├── loki-config.yml              # Log aggregation
│   ├── promtail-config.yml          # Log shipping
│   └── grafana/                     # Dashboards and config
└── scripts/
    ├── deploy.sh                    # Main deployment script
    ├── monitoring-setup.sh          # Monitoring configuration
    ├── disaster-recovery.sh         # Backup and recovery
    └── backup.sh                    # Automated backups
```

## Services and Ports

### Core Services
- **API Gateway**: 8080 (HTTP API endpoints)
- **Auth Service**: 8081 (Authentication and authorization)
- **User Service**: 8082 (User management)
- **File Service**: 8083 (File storage and processing)
- **Workflow Service**: 8084 (Cross-service orchestration)
- **Tenant Service**: 8085 (Multi-tenant management)

### Infrastructure Services
- **PostgreSQL**: 5432 (Primary database)
- **Redis**: 6379 (Caching and sessions)
- **Temporal**: 7233 (Workflow engine), 8088 (Web UI)

### Monitoring Services
- **Prometheus**: 9090 (Metrics collection)
- **Grafana**: 3001 (Dashboards and visualization)
- **Loki**: 3100 (Log aggregation)
- **Alertmanager**: 9093 (Alert management)

### External Access
- **HTTPS**: 443 (Main application access)
- **HTTP**: 80 (Redirects to HTTPS)

## Deployment Commands

### Initial Deployment
```bash
# Full production deployment
sudo ./scripts/deploy.sh deploy

# Health check only
sudo ./scripts/deploy.sh health

# Create backup before deployment
sudo ./scripts/deploy.sh backup
```

### Monitoring Setup
```bash
# Set up monitoring infrastructure
sudo ./scripts/monitoring-setup.sh

# Start with monitoring
sudo docker-compose -f docker-compose.prod.yml -f docker-compose.monitoring.yml up -d
```

### Backup and Recovery
```bash
# List available backups
sudo ./scripts/disaster-recovery.sh list

# Create manual backup
sudo ./scripts/disaster-recovery.sh backup

# Full system recovery
sudo ./scripts/disaster-recovery.sh full

# Partial recovery (database only)
sudo ./scripts/disaster-recovery.sh partial database
```

## Environment Configuration

### Required Environment Variables

```bash
# Database
POSTGRES_USER=adx_core_user
POSTGRES_PASSWORD=your_secure_password
POSTGRES_DB=adx_core_prod

# Redis
REDIS_PASSWORD=your_redis_password

# JWT Authentication
JWT_SECRET=your_jwt_secret_minimum_32_chars

# Storage (S3 recommended for production)
STORAGE_PROVIDER=s3
AWS_ACCESS_KEY_ID=your_aws_key
AWS_SECRET_ACCESS_KEY=your_aws_secret
AWS_REGION=us-east-1
S3_BUCKET=adx-core-files-prod

# Monitoring
GRAFANA_USER=admin
GRAFANA_PASSWORD=your_grafana_password

# Domain Configuration
DOMAIN=your-domain.com
API_DOMAIN=api.your-domain.com
```

### Optional Configuration

```bash
# Email (for notifications)
SMTP_HOST=smtp.your-provider.com
SMTP_PORT=587
SMTP_USER=your_smtp_user
SMTP_PASSWORD=your_smtp_password

# External Services
STRIPE_SECRET_KEY=sk_live_your_stripe_key
STRIPE_WEBHOOK_SECRET=whsec_your_webhook_secret

# Backup
BACKUP_S3_BUCKET=adx-core-backups-prod
BACKUP_RETENTION_DAYS=30
```

## SSL Certificate Setup

### Using Let's Encrypt (Recommended)
```bash
# Install certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificates
sudo certbot certonly --standalone -d api.your-domain.com -d monitoring.your-domain.com

# Copy certificates
sudo cp /etc/letsencrypt/live/api.your-domain.com/fullchain.pem nginx/ssl/cert.pem
sudo cp /etc/letsencrypt/live/api.your-domain.com/privkey.pem nginx/ssl/key.pem

# Set up auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

### Using Custom Certificates
```bash
# Place your certificates
cp your-certificate.pem nginx/ssl/cert.pem
cp your-private-key.pem nginx/ssl/key.pem

# Set proper permissions
chmod 600 nginx/ssl/key.pem
chmod 644 nginx/ssl/cert.pem
```

## Monitoring and Alerting

### Accessing Monitoring Dashboards

1. **Grafana**: https://monitoring.your-domain.com
   - Username: admin
   - Password: (from GRAFANA_PASSWORD env var)

2. **Prometheus**: https://monitoring.your-domain.com/prometheus
   - Direct access to metrics and queries

3. **Temporal UI**: https://monitoring.your-domain.com/temporal
   - Workflow monitoring and debugging

### Available Dashboards

- **ADX Core Services**: Service health, request rates, response times
- **Temporal Workflows**: Workflow execution, success rates, queue depth
- **Infrastructure**: CPU, memory, disk, network metrics
- **Database**: PostgreSQL and Redis performance metrics

### Alert Configuration

Alerts are configured in `monitoring/alert_rules.yml` and include:

- Service downtime detection
- High error rates (>10% 5xx responses)
- High response times (>1s 95th percentile)
- Resource usage (CPU >80%, Memory >1GB)
- Workflow failures and queue backlogs
- Database connection and performance issues

## Backup Strategy

### Automated Backups

- **Schedule**: Daily at 2:00 AM
- **Retention**: 30 days local, configurable S3 retention
- **Components**: Database, Redis, Configuration
- **Location**: `/opt/adx-core/backups/`

### Manual Backup

```bash
# Create immediate backup
sudo ./scripts/disaster-recovery.sh backup

# Verify backup integrity
sudo ./scripts/disaster-recovery.sh verify /path/to/backup
```

### Backup Verification

All backups are automatically verified for:
- Database dump integrity
- Configuration completeness
- File permissions and structure

## Disaster Recovery

### Recovery Time Objectives (RTO)

- **Database Recovery**: < 15 minutes
- **Full System Recovery**: < 30 minutes
- **Configuration Recovery**: < 5 minutes

### Recovery Procedures

1. **Full System Recovery**:
   ```bash
   sudo ./scripts/disaster-recovery.sh full
   ```

2. **Database Only Recovery**:
   ```bash
   sudo ./scripts/disaster-recovery.sh partial database
   ```

3. **Configuration Recovery**:
   ```bash
   sudo ./scripts/disaster-recovery.sh partial config
   ```

### Recovery Testing

Monthly recovery testing is recommended:

```bash
# Test recovery in staging environment
sudo ./scripts/disaster-recovery.sh verify
sudo ./scripts/disaster-recovery.sh full /path/to/test/backup
```

## Security Considerations

### Network Security

- All external traffic encrypted with TLS 1.2+
- Internal service communication over private Docker networks
- Rate limiting on all public endpoints
- CORS protection configured

### Access Control

- JWT-based authentication with configurable expiration
- Role-based access control (RBAC)
- Multi-tenant data isolation
- API key management for external integrations

### Data Protection

- Database encryption at rest
- Redis password protection
- Secure backup encryption
- PII data handling compliance

## Performance Tuning

### Database Optimization

```sql
-- Recommended PostgreSQL settings for production
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
```

### Redis Configuration

```
# Recommended Redis settings
maxmemory 512mb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
```

### Service Scaling

Each service can be scaled independently:

```bash
# Scale specific service
docker-compose -f docker-compose.prod.yml up -d --scale auth-service=3

# Scale with load balancer
# Update nginx upstream configuration
```

## Troubleshooting

### Common Issues

1. **Service Won't Start**:
   ```bash
   # Check logs
   docker-compose -f docker-compose.prod.yml logs service-name
   
   # Check health
   sudo ./scripts/deploy.sh health
   ```

2. **Database Connection Issues**:
   ```bash
   # Check PostgreSQL status
   docker exec adx-core-postgres-prod pg_isready -U $POSTGRES_USER
   
   # Check connections
   docker exec adx-core-postgres-prod psql -U $POSTGRES_USER -c "SELECT count(*) FROM pg_stat_activity;"
   ```

3. **Temporal Workflow Issues**:
   ```bash
   # Check Temporal health
   docker exec adx-core-temporal-prod tctl --address temporal:7233 cluster health
   
   # View workflow history
   docker exec adx-core-temporal-prod tctl --address temporal:7233 workflow list
   ```

### Log Locations

- **Application Logs**: `docker-compose logs service-name`
- **Deployment Logs**: `/var/log/adx-core-deploy.log`
- **Recovery Logs**: `/var/log/adx-core-recovery.log`
- **Backup Logs**: `/var/log/adx-core-backup.log`
- **System Logs**: `/var/log/syslog`

### Performance Monitoring

```bash
# Check resource usage
docker stats

# Check disk usage
df -h

# Check network connections
netstat -tulpn | grep :8080
```

## Maintenance

### Regular Maintenance Tasks

1. **Weekly**:
   - Review monitoring dashboards
   - Check backup integrity
   - Update security patches

2. **Monthly**:
   - Test disaster recovery procedures
   - Review and rotate logs
   - Performance optimization review

3. **Quarterly**:
   - Security audit
   - Capacity planning review
   - Update dependencies

### Update Procedures

```bash
# Update ADX Core services
git pull origin main
sudo ./scripts/deploy.sh deploy

# Update monitoring stack
docker-compose -f docker-compose.monitoring.yml pull
docker-compose -f docker-compose.monitoring.yml up -d
```

## Support and Documentation

- **API Documentation**: https://api.your-domain.com/docs
- **Temporal UI**: https://monitoring.your-domain.com/temporal
- **Monitoring**: https://monitoring.your-domain.com
- **Logs**: Accessible via Grafana Loki integration

For additional support, check the main project documentation and issue tracker.