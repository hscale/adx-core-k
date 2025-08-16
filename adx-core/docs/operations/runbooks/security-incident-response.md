# Security Incident Response Runbook

## Overview
This runbook provides procedures for responding to security incidents in the ADX Core production environment, including detection, containment, eradication, and recovery.

## Incident Classification

### Severity Levels

#### Critical (P0)
- Active data breach or exfiltration
- Ransomware or destructive malware
- Complete system compromise
- Unauthorized access to production systems

#### High (P1)
- Suspected data breach
- Privilege escalation
- Malware detection
- Unauthorized configuration changes

#### Medium (P2)
- Failed authentication attempts (brute force)
- Suspicious network activity
- Policy violations
- Vulnerability exploitation attempts

#### Low (P3)
- Security tool alerts
- Minor policy violations
- Informational security events

## Incident Response Team

### Core Team Roles
- **Incident Commander**: Overall response coordination
- **Security Lead**: Security analysis and containment
- **Technical Lead**: System recovery and remediation
- **Communications Lead**: Stakeholder communication
- **Legal/Compliance**: Regulatory and legal requirements

### Contact Information
- **Security Team**: security@adxcore.com, +1-XXX-XXX-XXXX
- **On-call Engineer**: +1-XXX-XXX-XXXX
- **Legal Team**: legal@adxcore.com
- **Management**: management@adxcore.com

## Detection and Analysis

### Automated Detection
```bash
# Check security alerts
kubectl logs deployment/security-monitor -n adx-core | grep "SECURITY_ALERT"

# Review failed authentication attempts
kubectl exec -it deployment/auth-service -n adx-core -- grep "authentication_failed" /var/log/auth.log

# Check for suspicious network activity
kubectl exec -it deployment/api-gateway -n adx-core -- netstat -an | grep ESTABLISHED
```

### Manual Investigation
```bash
# Check system integrity
kubectl exec -it deployment/api-gateway -n adx-core -- find /app -type f -exec sha256sum {} \; > current-checksums.txt
diff baseline-checksums.txt current-checksums.txt

# Review access logs
kubectl logs deployment/api-gateway -n adx-core | grep -E "(401|403|500)" | tail -100

# Check for privilege escalation
kubectl get pods -n adx-core -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.securityContext.runAsUser}{"\n"}{end}'
```

### Log Analysis
```bash
# Search for security events in Loki
curl -G -s "http://loki:3100/loki/api/v1/query" \
  --data-urlencode 'query={job=~".*"} |= "SECURITY" | json' \
  --data-urlencode 'limit=100'

# Check for SQL injection attempts
curl -G -s "http://loki:3100/loki/api/v1/query" \
  --data-urlencode 'query={job="api-gateway"} |~ "(?i)(union|select|drop|insert|update|delete).*--"' \
  --data-urlencode 'limit=50'

# Look for XSS attempts
curl -G -s "http://loki:3100/loki/api/v1/query" \
  --data-urlencode 'query={job="api-gateway"} |~ "(?i)(<script|javascript:|onerror=)"' \
  --data-urlencode 'limit=50'
```

## Containment Procedures

### Immediate Containment

#### Isolate Compromised Systems
```bash
# Isolate compromised pod
kubectl label pod compromised-pod-name quarantine=true -n adx-core
kubectl patch networkpolicy default-deny -n adx-core -p '{"spec":{"podSelector":{"matchLabels":{"quarantine":"true"}}}}'

# Block suspicious IP addresses
kubectl create configmap blocked-ips --from-literal=ips="192.168.1.100,10.0.0.50" -n adx-core
kubectl patch deployment api-gateway -n adx-core -p '{"spec":{"template":{"spec":{"containers":[{"name":"api-gateway","env":[{"name":"BLOCKED_IPS","valueFrom":{"configMapKeyRef":{"name":"blocked-ips","key":"ips"}}}]}]}}}}'

# Disable compromised user accounts
kubectl exec -it deployment/auth-service -n adx-core -- psql -d adx_core -c "UPDATE users SET status = 'suspended' WHERE id IN ('user1', 'user2');"
```

#### Emergency Service Shutdown
```bash
# Scale down compromised services
kubectl scale deployment/compromised-service --replicas=0 -n adx-core

# Enable maintenance mode
kubectl create configmap maintenance-mode --from-literal=enabled=true -n adx-core
kubectl set env deployment/api-gateway MAINTENANCE_MODE=true -n adx-core

# Redirect traffic to safe services only
kubectl patch service api-gateway -n adx-core -p '{"spec":{"selector":{"app":"api-gateway-safe"}}}'
```

### Evidence Preservation
```bash
# Create forensic snapshots
kubectl exec -it compromised-pod -n adx-core -- tar -czf /tmp/forensic-snapshot-$(date +%Y%m%d-%H%M%S).tar.gz /var/log /tmp /app

# Copy evidence to secure location
kubectl cp adx-core/compromised-pod:/tmp/forensic-snapshot-*.tar.gz ./evidence/

# Preserve memory dumps
kubectl exec -it compromised-pod -n adx-core -- gcore -o /tmp/memory-dump $(pgrep main-process)

# Export container logs
kubectl logs compromised-pod -n adx-core --previous > evidence/compromised-pod-logs.txt
```

### Network Isolation
```bash
# Create isolation network policy
cat > isolation-policy.yaml << EOF
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: isolate-compromised
  namespace: adx-core
spec:
  podSelector:
    matchLabels:
      quarantine: "true"
  policyTypes:
  - Ingress
  - Egress
  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53
EOF

kubectl apply -f isolation-policy.yaml
```

## Eradication and Recovery

### Malware Removal
```bash
# Scan for malware
kubectl exec -it deployment/security-scanner -n adx-core -- clamscan -r /app /var/log

# Remove identified threats
kubectl exec -it compromised-pod -n adx-core -- rm -f /tmp/malicious-file.sh

# Update antivirus definitions
kubectl exec -it deployment/security-scanner -n adx-core -- freshclam
```

### System Hardening
```bash
# Update all container images
kubectl set image deployment/api-gateway api-gateway=adx-core/api-gateway:latest-secure -n adx-core
kubectl set image deployment/auth-service auth-service=adx-core/auth-service:latest-secure -n adx-core

# Apply security patches
kubectl apply -f adx-core/infrastructure/kubernetes/security-patches.yaml

# Update security policies
kubectl apply -f adx-core/infrastructure/kubernetes/pod-security-policies.yaml
```

### Credential Rotation
```bash
# Rotate JWT secrets
kubectl create secret generic jwt-secret-new --from-literal=secret="$(openssl rand -base64 32)" -n adx-core
kubectl patch deployment api-gateway -n adx-core -p '{"spec":{"template":{"spec":{"containers":[{"name":"api-gateway","env":[{"name":"JWT_SECRET","valueFrom":{"secretKeyRef":{"name":"jwt-secret-new","key":"secret"}}}]}]}}}}'

# Rotate database passwords
NEW_DB_PASSWORD=$(openssl rand -base64 32)
kubectl exec -it deployment/postgresql -n adx-core -- psql -c "ALTER USER adx_core_user PASSWORD '$NEW_DB_PASSWORD';"
kubectl patch secret database-secret -n adx-core -p '{"data":{"password":"'$(echo -n $NEW_DB_PASSWORD | base64)'"}}'

# Rotate API keys
kubectl create secret generic api-keys-new --from-literal=s3-key="$(aws iam create-access-key --user-name adx-core-s3 --query 'AccessKey.AccessKeyId' --output text)" -n adx-core

# Force user password resets
kubectl exec -it deployment/auth-service -n adx-core -- psql -d adx_core -c "UPDATE users SET password_reset_required = true WHERE last_login < NOW() - INTERVAL '30 days';"
```

### System Recovery
```bash
# Restore from clean backups
./adx-core/scripts/backup/restore-database.sh -s clean-backup-pre-incident.sql.enc.gz

# Redeploy services with latest secure images
kubectl apply -f adx-core/infrastructure/kubernetes/microservices-deployment.yaml

# Verify system integrity
kubectl exec -it deployment/api-gateway -n adx-core -- /app/scripts/integrity-check.sh

# Run security validation
./adx-core/scripts/security/security-audit.sh
```

## Communication Procedures

### Internal Communication
```bash
# Create incident channel
curl -X POST "https://slack.com/api/conversations.create" \
  -H "Authorization: Bearer $SLACK_TOKEN" \
  -d '{"name":"incident-'$(date +%Y%m%d-%H%M)'","is_private":false}'

# Send initial notification
curl -X POST "$SLACK_WEBHOOK_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "🚨 SECURITY INCIDENT DETECTED 🚨",
    "attachments": [{
      "color": "danger",
      "fields": [{
        "title": "Severity",
        "value": "P1 - High",
        "short": true
      }, {
        "title": "Status",
        "value": "Investigating",
        "short": true
      }]
    }]
  }'
```

### External Communication
```bash
# Update status page
curl -X POST "https://api.statuspage.io/v1/pages/$PAGE_ID/incidents" \
  -H "Authorization: OAuth $STATUSPAGE_TOKEN" \
  -d '{
    "incident": {
      "name": "Security Investigation",
      "status": "investigating",
      "impact_override": "minor",
      "body": "We are investigating a potential security issue and taking precautionary measures."
    }
  }'

# Prepare customer notification (if required)
cat > customer-notification.txt << EOF
Subject: Security Update - ADX Core

Dear Valued Customer,

We are writing to inform you of a security incident that may have affected your account. We detected suspicious activity on [DATE] and immediately took steps to secure our systems.

What happened:
[Brief description of incident]

What we're doing:
- Immediately secured affected systems
- Conducting thorough investigation
- Working with security experts
- Implementing additional safeguards

What you should do:
- Change your password as a precaution
- Review your account activity
- Enable two-factor authentication
- Contact us with any concerns

We sincerely apologize for any inconvenience and are committed to maintaining the security of your data.

Best regards,
ADX Core Security Team
EOF
```

### Regulatory Notification
```bash
# Prepare breach notification (if required)
cat > breach-notification.txt << EOF
GDPR/CCPA Breach Notification

Incident Details:
- Date of discovery: $(date)
- Nature of breach: [Description]
- Data categories affected: [List]
- Number of individuals affected: [Number]
- Potential consequences: [Assessment]

Measures taken:
- Immediate containment
- System hardening
- User notification
- Regulatory reporting

Contact: security@adxcore.com
EOF
```

## Forensic Analysis

### Digital Forensics
```bash
# Create forensic image
kubectl exec -it compromised-pod -n adx-core -- dd if=/dev/sda of=/tmp/forensic-image.dd bs=4096

# Calculate hash for integrity
kubectl exec -it compromised-pod -n adx-core -- sha256sum /tmp/forensic-image.dd > /tmp/forensic-image.sha256

# Analyze file system
kubectl exec -it forensic-workstation -n adx-core -- sleuthkit-tools /tmp/forensic-image.dd

# Extract artifacts
kubectl exec -it forensic-workstation -n adx-core -- volatility -f /tmp/memory-dump.raw --profile=LinuxUbuntu2004x64 linux_bash
```

### Log Analysis
```bash
# Correlate events across services
kubectl exec -it deployment/log-analyzer -n adx-core -- python3 /scripts/correlate-events.py \
  --start-time "2024-01-15 10:00:00" \
  --end-time "2024-01-15 12:00:00" \
  --services "api-gateway,auth-service,user-service"

# Generate timeline
kubectl exec -it deployment/log-analyzer -n adx-core -- python3 /scripts/generate-timeline.py \
  --incident-id "INC-20240115-001" \
  --output /tmp/incident-timeline.json

# Identify attack vectors
kubectl exec -it deployment/log-analyzer -n adx-core -- grep -E "(injection|xss|csrf|traversal)" /var/log/security.log
```

### Network Analysis
```bash
# Analyze network traffic
kubectl exec -it deployment/network-analyzer -n adx-core -- tcpdump -r /tmp/network-capture.pcap -nn

# Check for data exfiltration
kubectl exec -it deployment/network-analyzer -n adx-core -- python3 /scripts/detect-exfiltration.py \
  --pcap /tmp/network-capture.pcap \
  --threshold 10MB

# Identify command and control traffic
kubectl exec -it deployment/network-analyzer -n adx-core -- python3 /scripts/detect-c2.py \
  --logs /var/log/network.log \
  --indicators /etc/iocs.txt
```

## Post-Incident Activities

### Lessons Learned
```bash
# Schedule post-incident review
echo "Post-incident review scheduled for $(date -d '+3 days')" >> incident-log.txt

# Document findings
cat > lessons-learned.md << EOF
# Lessons Learned - Incident $(date +%Y%m%d)

## What Went Well
- Quick detection and response
- Effective containment procedures
- Good team coordination

## What Could Be Improved
- Earlier detection capabilities
- Faster containment procedures
- Better communication processes

## Action Items
- [ ] Implement additional monitoring
- [ ] Update incident response procedures
- [ ] Conduct security training
- [ ] Review access controls

## Timeline
- Detection: $(date -d '2 hours ago')
- Containment: $(date -d '1 hour ago')
- Recovery: $(date)
EOF
```

### Security Improvements
```bash
# Implement additional monitoring
kubectl apply -f adx-core/infrastructure/monitoring/enhanced-security-monitoring.yaml

# Update security policies
kubectl apply -f adx-core/infrastructure/kubernetes/updated-security-policies.yaml

# Deploy additional security tools
kubectl apply -f adx-core/infrastructure/security/intrusion-detection.yaml

# Schedule security training
curl -X POST "$TRAINING_API" \
  -H "Content-Type: application/json" \
  -d '{
    "course": "Security Incident Response",
    "participants": ["all-engineering"],
    "deadline": "'$(date -d '+30 days' +%Y-%m-%d)'"
  }'
```

### Compliance Reporting
```bash
# Generate compliance report
kubectl exec -it deployment/compliance-reporter -n adx-core -- python3 /scripts/generate-compliance-report.py \
  --incident-id "INC-$(date +%Y%m%d)-001" \
  --format "gdpr,ccpa,sox" \
  --output /tmp/compliance-report.pdf

# Submit regulatory notifications
curl -X POST "$REGULATORY_API" \
  -H "Authorization: Bearer $REGULATORY_TOKEN" \
  -F "report=@/tmp/compliance-report.pdf" \
  -F "incident_type=data_breach" \
  -F "notification_date=$(date +%Y-%m-%d)"
```

## Testing and Validation

### Incident Response Testing
```bash
# Schedule tabletop exercises
echo "Tabletop exercise scheduled for $(date -d 'first monday of next month')" >> training-schedule.txt

# Run simulated incidents
kubectl apply -f adx-core/infrastructure/testing/simulated-security-incident.yaml

# Test communication procedures
curl -X POST "$SLACK_WEBHOOK_URL" \
  -d '{"text":"🧪 SECURITY DRILL - This is a test of our incident response procedures"}'

# Validate backup and recovery procedures
./adx-core/scripts/backup/test-restore.sh --dry-run
```

### Security Validation
```bash
# Run penetration tests
kubectl apply -f adx-core/infrastructure/testing/penetration-test.yaml

# Validate security controls
./adx-core/scripts/security/security-audit.sh --full-scan

# Test incident detection
kubectl exec -it deployment/security-tester -n adx-core -- python3 /scripts/test-detection.py \
  --test-type "sql_injection,xss,brute_force" \
  --target "http://api-gateway:8080"
```

## Emergency Contacts

### Internal Contacts
- **Security Team Lead**: +1-XXX-XXX-XXXX
- **CISO**: +1-XXX-XXX-XXXX
- **Legal Counsel**: +1-XXX-XXX-XXXX
- **PR/Communications**: +1-XXX-XXX-XXXX

### External Contacts
- **FBI Cyber Division**: +1-855-292-3937
- **CISA**: +1-888-282-0870
- **Security Vendor Support**: +1-XXX-XXX-XXXX
- **Forensics Consultant**: +1-XXX-XXX-XXXX

### Regulatory Contacts
- **Data Protection Authority**: +XX-XXX-XXX-XXXX
- **Industry Regulator**: +XX-XXX-XXX-XXXX
- **Cyber Insurance**: +1-XXX-XXX-XXXX

## Related Documentation
- [Service Deployment Runbook](./service-deployment.md)
- [Monitoring Runbook](./monitoring.md)
- [Disaster Recovery Runbook](./disaster-recovery.md)
- [Production Deployment Guide](../production-deployment-guide.md)