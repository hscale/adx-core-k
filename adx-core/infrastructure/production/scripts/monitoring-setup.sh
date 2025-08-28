#!/bin/bash

# ADX Core Monitoring Setup Script
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRODUCTION_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MONITORING_DIR="$PRODUCTION_DIR/monitoring"

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

setup_grafana_dashboards() {
    log "Setting up Grafana dashboards..."
    
    local dashboards_dir="$MONITORING_DIR/grafana/dashboards"
    mkdir -p "$dashboards_dir"
    
    # ADX Core Services Dashboard
    cat > "$dashboards_dir/adx-core-services.json" << 'EOF'
{
  "dashboard": {
    "id": null,
    "title": "ADX Core Services",
    "tags": ["adx-core"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Service Health",
        "type": "stat",
        "targets": [
          {
            "expr": "up{job=~\"api-gateway|auth-service|user-service|file-service|workflow-service|tenant-service\"}",
            "legendFormat": "{{job}}"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "green", "value": 1}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{job}} - {{method}}"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
      },
      {
        "id": 3,
        "title": "Response Time (95th percentile)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "{{job}}"
          }
        ],
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 8}
      },
      {
        "id": 4,
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
            "legendFormat": "{{job}} - 5xx errors"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16}
      },
      {
        "id": 5,
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "process_resident_memory_bytes / 1024 / 1024",
            "legendFormat": "{{job}} (MB)"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16}
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "30s"
  }
}
EOF

    # Temporal Workflows Dashboard
    cat > "$dashboards_dir/temporal-workflows.json" << 'EOF'
{
  "dashboard": {
    "id": null,
    "title": "Temporal Workflows",
    "tags": ["temporal", "workflows"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Workflow Execution Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(temporal_workflow_started_total[5m])",
            "legendFormat": "Started - {{workflow_type}}"
          },
          {
            "expr": "rate(temporal_workflow_completed_total[5m])",
            "legendFormat": "Completed - {{workflow_type}}"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Workflow Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(temporal_workflow_completed_total{status=\"completed\"}[5m]) / rate(temporal_workflow_completed_total[5m]) * 100",
            "legendFormat": "Success Rate %"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 90},
                {"color": "green", "value": 95}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
      },
      {
        "id": 3,
        "title": "Workflow Queue Depth",
        "type": "graph",
        "targets": [
          {
            "expr": "temporal_workflow_task_queue_depth",
            "legendFormat": "{{task_queue}}"
          }
        ],
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 8}
      },
      {
        "id": 4,
        "title": "Workflow Execution Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(temporal_workflow_execution_time_seconds_bucket[5m]))",
            "legendFormat": "95th percentile - {{workflow_type}}"
          },
          {
            "expr": "histogram_quantile(0.50, rate(temporal_workflow_execution_time_seconds_bucket[5m]))",
            "legendFormat": "50th percentile - {{workflow_type}}"
          }
        ],
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 16}
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "30s"
  }
}
EOF

    # Infrastructure Dashboard
    cat > "$dashboards_dir/infrastructure.json" << 'EOF'
{
  "dashboard": {
    "id": null,
    "title": "Infrastructure",
    "tags": ["infrastructure"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "CPU Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "100 - (avg by (instance) (rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)",
            "legendFormat": "{{instance}}"
          }
        ],
        "yAxes": [
          {"unit": "percent", "max": 100}
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100",
            "legendFormat": "{{instance}}"
          }
        ],
        "yAxes": [
          {"unit": "percent", "max": 100}
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
      },
      {
        "id": 3,
        "title": "Disk Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "(1 - (node_filesystem_free_bytes / node_filesystem_size_bytes)) * 100",
            "legendFormat": "{{instance}} - {{mountpoint}}"
          }
        ],
        "yAxes": [
          {"unit": "percent", "max": 100}
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8}
      },
      {
        "id": 4,
        "title": "Network I/O",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(node_network_receive_bytes_total[5m])",
            "legendFormat": "{{instance}} - {{device}} RX"
          },
          {
            "expr": "rate(node_network_transmit_bytes_total[5m])",
            "legendFormat": "{{instance}} - {{device}} TX"
          }
        ],
        "yAxes": [
          {"unit": "bytes"}
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8}
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "30s"
  }
}
EOF

    log "Grafana dashboards created successfully"
}

setup_alertmanager() {
    log "Setting up Alertmanager configuration..."
    
    cat > "$MONITORING_DIR/alertmanager.yml" << 'EOF'
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alerts@your-domain.com'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
  - name: 'web.hook'
    email_configs:
      - to: 'admin@your-domain.com'
        subject: 'ADX Core Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          {{ end }}
    webhook_configs:
      - url: 'http://localhost:5001/webhook'
        send_resolved: true

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'dev', 'instance']
EOF

    log "Alertmanager configuration created"
}

create_monitoring_docker_compose() {
    log "Creating monitoring docker-compose override..."
    
    cat > "$PRODUCTION_DIR/docker-compose.monitoring.yml" << 'EOF'
version: '3.8'

services:
  # Node Exporter for system metrics
  node-exporter:
    image: prom/node-exporter:latest
    container_name: adx-core-node-exporter
    command:
      - '--path.rootfs=/host'
    volumes:
      - '/:/host:ro,rslave'
    networks:
      - monitoring
    ports:
      - "9100:9100"
    restart: unless-stopped

  # Docker metrics exporter
  docker-exporter:
    image: prometheusnet/docker_exporter
    container_name: adx-core-docker-exporter
    volumes:
      - '/var/run/docker.sock:/var/run/docker.sock'
    networks:
      - monitoring
    ports:
      - "9323:9323"
    restart: unless-stopped

  # Alertmanager
  alertmanager:
    image: prom/alertmanager:latest
    container_name: adx-core-alertmanager
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
      - '--storage.path=/alertmanager'
      - '--web.external-url=http://localhost:9093'
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml
    networks:
      - monitoring
    ports:
      - "9093:9093"
    restart: unless-stopped

  # Postgres Exporter
  postgres-exporter:
    image: prometheuscommunity/postgres-exporter
    container_name: adx-core-postgres-exporter
    environment:
      DATA_SOURCE_NAME: "postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/adx_core_prod?sslmode=disable"
    networks:
      - adx-core-prod
      - monitoring
    ports:
      - "9187:9187"
    restart: unless-stopped
    depends_on:
      - postgres

  # Redis Exporter
  redis-exporter:
    image: oliver006/redis_exporter
    container_name: adx-core-redis-exporter
    environment:
      REDIS_ADDR: "redis://redis:6379"
      REDIS_PASSWORD: "${REDIS_PASSWORD}"
    networks:
      - adx-core-prod
      - monitoring
    ports:
      - "9121:9121"
    restart: unless-stopped
    depends_on:
      - redis
EOF

    log "Monitoring docker-compose override created"
}

setup_log_rotation() {
    log "Setting up log rotation..."
    
    cat > "/etc/logrotate.d/adx-core" << 'EOF'
/var/log/adx-core/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 root root
    postrotate
        docker kill -s USR1 $(docker ps -q --filter name=adx-core)
    endscript
}

/var/log/adx-core-deploy.log {
    weekly
    missingok
    rotate 12
    compress
    delaycompress
    notifempty
    create 644 root root
}
EOF

    log "Log rotation configured"
}

create_backup_script() {
    log "Creating automated backup script..."
    
    cat > "$PRODUCTION_DIR/scripts/backup.sh" << 'EOF'
#!/bin/bash

# ADX Core Automated Backup Script
set -e

BACKUP_DIR="/opt/adx-core/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/automated_$TIMESTAMP"
RETENTION_DAYS=30

# Create backup directory
mkdir -p "$BACKUP_PATH"

# Backup database
echo "Backing up database..."
docker exec adx-core-postgres-prod pg_dump -U "$POSTGRES_USER" adx_core_prod | gzip > "$BACKUP_PATH/database.sql.gz"

# Backup Redis
echo "Backing up Redis..."
docker exec adx-core-redis-prod redis-cli --rdb - | gzip > "$BACKUP_PATH/redis.rdb.gz"

# Backup configuration
echo "Backing up configuration..."
tar -czf "$BACKUP_PATH/config.tar.gz" -C /opt/adx-core/infrastructure/production .

# Upload to S3 if configured
if [[ -n "$BACKUP_S3_BUCKET" ]]; then
    echo "Uploading backup to S3..."
    aws s3 cp "$BACKUP_PATH" "s3://$BACKUP_S3_BUCKET/backups/$(basename $BACKUP_PATH)/" --recursive
fi

# Clean up old backups
echo "Cleaning up old backups..."
find "$BACKUP_DIR" -name "automated_*" -type d -mtime +$RETENTION_DAYS -exec rm -rf {} +

echo "Backup completed: $BACKUP_PATH"
EOF

    chmod +x "$PRODUCTION_DIR/scripts/backup.sh"
    
    # Create cron job for daily backups
    cat > "/etc/cron.d/adx-core-backup" << EOF
# ADX Core daily backup
0 2 * * * root $PRODUCTION_DIR/scripts/backup.sh >> /var/log/adx-core-backup.log 2>&1
EOF

    log "Automated backup script created and scheduled"
}

main() {
    log "Setting up ADX Core monitoring and operational tools..."
    
    setup_grafana_dashboards
    setup_alertmanager
    create_monitoring_docker_compose
    setup_log_rotation
    create_backup_script
    
    log "Monitoring setup completed successfully!"
    log "Next steps:"
    log "1. Configure your .env file with proper credentials"
    log "2. Run: docker-compose -f docker-compose.prod.yml -f docker-compose.monitoring.yml up -d"
    log "3. Access Grafana at https://monitoring.your-domain.com"
    log "4. Access Prometheus at https://monitoring.your-domain.com/prometheus"
    log "5. Access Temporal UI at https://monitoring.your-domain.com/temporal"
}

main "$@"
EOF