# ADX CORE - Infrastructure and Deployment Architecture

## Overview

ADX CORE implements a cloud-native, highly available infrastructure architecture designed for enterprise-scale deployment across multiple environments. The infrastructure supports hybrid cloud, multi-region deployment, and comprehensive disaster recovery capabilities.

## Infrastructure Architecture

### Multi-Cloud Architecture
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              Global Load Balancer                              │
│                            (Cloudflare / AWS Route 53)                         │
└─────────────────────────────────────────────────────────────────────────────────┘
                                        │
        ┌───────────────────────────────┼───────────────────────────────┐
        │                               │                               │
┌───────────────────┐        ┌───────────────────┐        ┌───────────────────┐
│   Primary Region  │        │  Secondary Region │        │  Tertiary Region  │
│    (US-East-1)    │        │   (EU-West-1)     │        │   (AP-South-1)    │
│                   │        │                   │        │                   │
│ ┌───────────────┐ │        │ ┌───────────────┐ │        │ ┌───────────────┐ │
│ │ Kubernetes    │ │        │ │ Kubernetes    │ │        │ │ Kubernetes    │ │
│ │ Cluster       │ │        │ │ Cluster       │ │        │ │ Cluster       │ │
│ │ (Production)  │ │        │ │ (Hot Standby) │ │        │ │ (DR Site)     │ │
│ └───────────────┘ │        │ └───────────────┘ │        │ └───────────────┘ │
│                   │        │                   │        │                   │
│ ┌───────────────┐ │        │ ┌───────────────┐ │        │ ┌───────────────┐ │
│ │ Database      │ │        │ │ Database      │ │        │ │ Database      │ │
│ │ (Primary)     │◄┼────────┼►│ (Read Replica)│ │        │ │ (Backup)      │ │
│ └───────────────┘ │        │ └───────────────┘ │        │ └───────────────┘ │
│                   │        │                   │        │                   │
│ ┌───────────────┐ │        │ ┌───────────────┐ │        │ ┌───────────────┐ │
│ │ File Storage  │ │        │ │ File Storage  │ │        │ │ File Storage  │ │
│ │ (Primary)     │◄┼────────┼►│ (Replica)     │ │        │ │ (Backup)      │ │
│ └───────────────┘ │        │ └───────────────┘ │        │ └───────────────┘ │
└───────────────────┘        └───────────────────┘        └───────────────────┘
```

### Kubernetes Cluster Architecture
```yaml
# Production Kubernetes cluster configuration
apiVersion: v1
kind: Namespace
metadata:
  name: adx-core-production
  labels:
    environment: production
    tier: application

---
# Node pools configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: cluster-config
  namespace: adx-core-production
data:
  cluster.yaml: |
    nodeGroups:
      - name: system-nodes
        instanceType: m5.large
        minSize: 3
        maxSize: 6
        desiredCapacity: 3
        labels:
          node-type: system
        taints:
          - key: node-type
            value: system
            effect: NoSchedule
        
      - name: application-nodes
        instanceType: c5.2xlarge
        minSize: 6
        maxSize: 50
        desiredCapacity: 10
        labels:
          node-type: application
        
      - name: ai-nodes
        instanceType: p3.2xlarge
        minSize: 0
        maxSize: 10
        desiredCapacity: 2
        labels:
          node-type: ai-workload
          gpu: nvidia-v100
        taints:
          - key: nvidia.com/gpu
            value: "true"
            effect: NoSchedule
        
      - name: database-nodes
        instanceType: r5.4xlarge
        minSize: 3
        maxSize: 9
        desiredCapacity: 3
        labels:
          node-type: database
        taints:
          - key: node-type
            value: database
            effect: NoSchedule

---
# Cluster autoscaler configuration
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cluster-autoscaler
  namespace: kube-system
spec:
  replicas: 1
  selector:
    matchLabels:
      app: cluster-autoscaler
  template:
    metadata:
      labels:
        app: cluster-autoscaler
    spec:
      serviceAccountName: cluster-autoscaler
      containers:
      - image: k8s.gcr.io/autoscaling/cluster-autoscaler:v1.21.0
        name: cluster-autoscaler
        resources:
          limits:
            cpu: 100m
            memory: 300Mi
          requests:
            cpu: 100m
            memory: 300Mi
        command:
        - ./cluster-autoscaler
        - --v=4
        - --stderrthreshold=info
        - --cloud-provider=aws
        - --skip-nodes-with-local-storage=false
        - --expander=least-waste
        - --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/adx-core-production
        - --balance-similar-node-groups
        - --scale-down-enabled=true
        - --scale-down-delay-after-add=10m
        - --scale-down-unneeded-time=10m
        - --scale-down-utilization-threshold=0.5
        env:
        - name: AWS_REGION
          value: us-east-1
```

### Service Mesh Architecture
```yaml
# Istio service mesh configuration
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: adx-core-istio
spec:
  values:
    global:
      meshID: adx-core-mesh
      multiCluster:
        clusterName: adx-core-production
      network: adx-core-network
  components:
    pilot:
      k8s:
        resources:
          requests:
            cpu: 500m
            memory: 2048Mi
        hpaSpec:
          minReplicas: 2
          maxReplicas: 10
          metrics:
          - type: Resource
            resource:
              name: cpu
              target:
                type: Utilization
                averageUtilization: 80
    
    ingressGateways:
    - name: istio-ingressgateway
      enabled: true
      k8s:
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
        hpaSpec:
          minReplicas: 3
          maxReplicas: 20
        service:
          type: LoadBalancer
          annotations:
            service.beta.kubernetes.io/aws-load-balancer-type: nlb
            service.beta.kubernetes.io/aws-load-balancer-cross-zone-load-balancing-enabled: "true"
    
    egressGateways:
    - name: istio-egressgateway
      enabled: true
      k8s:
        resources:
          requests:
            cpu: 100m
            memory: 128Mi

---
# Traffic management
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: adx-core-gateway
  namespace: adx-core-production
spec:
  selector:
    istio: ingressgateway
  servers:
  - port:
      number: 443
      name: https
      protocol: HTTPS
    tls:
      mode: SIMPLE
      credentialName: adx-core-tls
    hosts:
    - api.adxcore.com
    - "*.adxcore.com"
  - port:
      number: 80
      name: http
      protocol: HTTP
    hosts:
    - api.adxcore.com
    - "*.adxcore.com"
    tls:
      httpsRedirect: true

---
# Virtual service for traffic routing
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: adx-core-vs
  namespace: adx-core-production
spec:
  hosts:
  - api.adxcore.com
  gateways:
  - adx-core-gateway
  http:
  - match:
    - uri:
        prefix: /api/v1/
    route:
    - destination:
        host: adx-core-api
        port:
          number: 80
      weight: 90
    - destination:
        host: adx-core-api-canary
        port:
          number: 80
      weight: 10
    fault:
      delay:
        percentage:
          value: 0.1
        fixedDelay: 5s
    retries:
      attempts: 3
      perTryTimeout: 10s
      retryOn: 5xx,reset,connect-failure,refused-stream
    timeout: 30s
```

### Database Infrastructure
```yaml
# PostgreSQL cluster with high availability
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: adx-core-postgres
  namespace: adx-core-production
spec:
  instances: 3
  
  postgresql:
    parameters:
      max_connections: "500"
      shared_buffers: "256MB"
      effective_cache_size: "1GB"
      maintenance_work_mem: "64MB"
      checkpoint_completion_target: "0.9"
      wal_buffers: "16MB"
      default_statistics_target: "100"
      random_page_cost: "1.1"
      effective_io_concurrency: "200"
      work_mem: "4MB"
      min_wal_size: "1GB"
      max_wal_size: "4GB"
      max_worker_processes: "8"
      max_parallel_workers_per_gather: "4"
      max_parallel_workers: "8"
      max_parallel_maintenance_workers: "4"
  
  bootstrap:
    initdb:
      database: adx_core
      owner: adx_core_user
      secret:
        name: adx-core-postgres-credentials
  
  storage:
    size: 1Ti
    storageClass: fast-ssd
  
  resources:
    requests:
      memory: "2Gi"
      cpu: "1000m"
    limits:
      memory: "4Gi"
      cpu: "2000m"
  
  monitoring:
    enabled: true
    prometheusRule:
      enabled: true
  
  backup:
    retentionPolicy: "30d"
    barmanObjectStore:
      destinationPath: "s3://adx-core-backups/postgres"
      s3Credentials:
        accessKeyId:
          name: backup-credentials
          key: ACCESS_KEY_ID
        secretAccessKey:
          name: backup-credentials
          key: SECRET_ACCESS_KEY
      wal:
        retention: "5d"
      data:
        retention: "30d"
        jobs: 2

---
# Redis cluster for caching
apiVersion: redis.redis.opstreelabs.in/v1beta1
kind: RedisCluster
metadata:
  name: adx-core-redis
  namespace: adx-core-production
spec:
  clusterSize: 6
  clusterVersion: v7.0
  persistenceEnabled: true
  
  redisExporter:
    enabled: true
    image: oliver006/redis_exporter:latest
  
  storage:
    volumeClaimTemplate:
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 100Gi
        storageClassName: fast-ssd
  
  resources:
    requests:
      cpu: 500m
      memory: 1Gi
    limits:
      cpu: 1000m
      memory: 2Gi
  
  securityContext:
    runAsUser: 1000
    fsGroup: 1000
  
  redisConfig:
    maxmemory: "1gb"
    maxmemory-policy: "allkeys-lru"
    save: "900 1 300 10 60 10000"
    tcp-keepalive: "60"
    timeout: "300"
```

### Monitoring and Observability Stack
```yaml
# Prometheus monitoring stack
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s
      external_labels:
        cluster: adx-core-production
        region: us-east-1
    
    rule_files:
      - "/etc/prometheus/rules/*.yml"
    
    alerting:
      alertmanagers:
        - static_configs:
            - targets:
              - alertmanager:9093
    
    scrape_configs:
      - job_name: 'kubernetes-apiservers'
        kubernetes_sd_configs:
        - role: endpoints
        scheme: https
        tls_config:
          ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
        bearer_token_file: /var/run/secrets/kubernetes.io/serviceaccount/token
        relabel_configs:
        - source_labels: [__meta_kubernetes_namespace, __meta_kubernetes_service_name, __meta_kubernetes_endpoint_port_name]
          action: keep
          regex: default;kubernetes;https
      
      - job_name: 'kubernetes-nodes'
        kubernetes_sd_configs:
        - role: node
        scheme: https
        tls_config:
          ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
        bearer_token_file: /var/run/secrets/kubernetes.io/serviceaccount/token
        relabel_configs:
        - action: labelmap
          regex: __meta_kubernetes_node_label_(.+)
        - target_label: __address__
          replacement: kubernetes.default.svc:443
        - source_labels: [__meta_kubernetes_node_name]
          regex: (.+)
          target_label: __metrics_path__
          replacement: /api/v1/nodes/${1}/proxy/metrics
      
      - job_name: 'adx-core-services'
        kubernetes_sd_configs:
        - role: endpoints
          namespaces:
            names:
            - adx-core-production
        relabel_configs:
        - source_labels: [__meta_kubernetes_service_annotation_prometheus_io_scrape]
          action: keep
          regex: true
        - source_labels: [__meta_kubernetes_service_annotation_prometheus_io_path]
          action: replace
          target_label: __metrics_path__
          regex: (.+)
        - source_labels: [__address__, __meta_kubernetes_service_annotation_prometheus_io_port]
          action: replace
          regex: ([^:]+)(?::\d+)?;(\d+)
          replacement: $1:$2
          target_label: __address__
        - action: labelmap
          regex: __meta_kubernetes_service_label_(.+)
        - source_labels: [__meta_kubernetes_namespace]
          action: replace
          target_label: kubernetes_namespace
        - source_labels: [__meta_kubernetes_service_name]
          action: replace
          target_label: kubernetes_name

---
# Grafana dashboard configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: grafana-dashboards
  namespace: monitoring
data:
  adx-core-overview.json: |
    {
      "dashboard": {
        "id": null,
        "title": "ADX CORE - System Overview",
        "tags": ["adx-core", "overview"],
        "timezone": "browser",
        "panels": [
          {
            "id": 1,
            "title": "Request Rate",
            "type": "graph",
            "targets": [
              {
                "expr": "sum(rate(http_requests_total{job=\"adx-core-api\"}[5m])) by (method, status)",
                "legendFormat": "{{method}} - {{status}}"
              }
            ],
            "yAxes": [
              {
                "label": "Requests/sec",
                "min": 0
              }
            ]
          },
          {
            "id": 2,
            "title": "Response Time",
            "type": "graph",
            "targets": [
              {
                "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket{job=\"adx-core-api\"}[5m])) by (le))",
                "legendFormat": "95th percentile"
              },
              {
                "expr": "histogram_quantile(0.50, sum(rate(http_request_duration_seconds_bucket{job=\"adx-core-api\"}[5m])) by (le))",
                "legendFormat": "50th percentile"
              }
            ]
          },
          {
            "id": 3,
            "title": "Error Rate",
            "type": "singlestat",
            "targets": [
              {
                "expr": "sum(rate(http_requests_total{job=\"adx-core-api\",status=~\"5..\"}[5m])) / sum(rate(http_requests_total{job=\"adx-core-api\"}[5m])) * 100",
                "legendFormat": "Error Rate %"
              }
            ],
            "thresholds": "1,5",
            "colorBackground": true
          }
        ]
      }
    }

---
# Jaeger tracing configuration
apiVersion: jaegertracing.io/v1
kind: Jaeger
metadata:
  name: adx-core-jaeger
  namespace: monitoring
spec:
  strategy: production
  
  collector:
    maxReplicas: 5
    resources:
      limits:
        cpu: 1000m
        memory: 1Gi
      requests:
        cpu: 500m
        memory: 512Mi
  
  query:
    replicas: 2
    resources:
      limits:
        cpu: 500m
        memory: 512Mi
      requests:
        cpu: 250m
        memory: 256Mi
  
  storage:
    type: elasticsearch
    elasticsearch:
      nodeCount: 3
      storage:
        size: 100Gi
        storageClassName: fast-ssd
      resources:
        requests:
          cpu: 1000m
          memory: 2Gi
        limits:
          cpu: 2000m
          memory: 4Gi
```

### Security Infrastructure
```yaml
# Network policies for security
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: adx-core-network-policy
  namespace: adx-core-production
spec:
  podSelector:
    matchLabels:
      app: adx-core
  policyTypes:
  - Ingress
  - Egress
  
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: istio-system
    - podSelector:
        matchLabels:
          app: istio-proxy
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 9090
  
  - from:
    - podSelector:
        matchLabels:
          app: adx-core-api
    ports:
    - protocol: TCP
      port: 8080
  
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  
  - to: []
    ports:
    - protocol: TCP
      port: 443
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53

---
# Pod Security Policy
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: adx-core-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  
  requiredDropCapabilities:
    - ALL
  
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  
  runAsUser:
    rule: 'MustRunAsNonRoot'
  
  seLinux:
    rule: 'RunAsAny'
  
  fsGroup:
    rule: 'RunAsAny'
  
  readOnlyRootFilesystem: true

---
# RBAC configuration
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: adx-core-production
  name: adx-core-role
rules:
- apiGroups: [""]
  resources: ["pods", "services", "endpoints", "configmaps", "secrets"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["extensions"]
  resources: ["ingresses"]
  verbs: ["get", "list", "watch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: adx-core-rolebinding
  namespace: adx-core-production
subjects:
- kind: ServiceAccount
  name: adx-core-serviceaccount
  namespace: adx-core-production
roleRef:
  kind: Role
  name: adx-core-role
  apiGroup: rbac.authorization.k8s.io
```

## CI/CD Pipeline Architecture

### GitOps Workflow
```yaml
# ArgoCD application configuration
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: adx-core-production
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: adx-core
  
  source:
    repoURL: https://github.com/adxcore/k8s-manifests
    targetRevision: HEAD
    path: overlays/production
    
    kustomize:
      images:
      - adxcore/api:v1.0.0
      - adxcore/worker:v1.0.0
      - adxcore/scheduler:v1.0.0
  
  destination:
    server: https://kubernetes.default.svc
    namespace: adx-core-production
  
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false
    
    syncOptions:
    - CreateNamespace=true
    - PrunePropagationPolicy=foreground
    - PruneLast=true
    - RespectIgnoreDifferences=true
    
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
  
  revisionHistoryLimit: 10

---
# Progressive delivery with Argo Rollouts
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: adx-core-api
  namespace: adx-core-production
spec:
  replicas: 10
  
  strategy:
    canary:
      maxSurge: "25%"
      maxUnavailable: 0
      
      analysis:
        templates:
        - templateName: success-rate
        - templateName: latency
        startingStep: 2
        args:
        - name: service-name
          value: adx-core-api-canary
      
      steps:
      - setWeight: 10
      - pause:
          duration: 1m
      - setWeight: 20
      - pause:
          duration: 2m
      - setWeight: 40
      - pause:
          duration: 5m
      - setWeight: 60
      - pause:
          duration: 10m
      - setWeight: 80
      - pause:
          duration: 10m
      
      trafficRouting:
        istio:
          virtualService:
            name: adx-core-vs
            routes:
            - primary
          destinationRule:
            name: adx-core-dr
            canarySubsetName: canary
            stableSubsetName: stable
  
  selector:
    matchLabels:
      app: adx-core-api
  
  template:
    metadata:
      labels:
        app: adx-core-api
    spec:
      containers:
      - name: api
        image: adxcore/api:v1.0.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5

---
# Analysis templates for progressive delivery
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
  namespace: adx-core-production
spec:
  args:
  - name: service-name
  
  metrics:
  - name: success-rate
    interval: 1m
    count: 5
    successCondition: result[0] >= 0.95
    failureLimit: 3
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          sum(rate(http_requests_total{service="{{args.service-name}}",status!~"5.."}[2m])) /
          sum(rate(http_requests_total{service="{{args.service-name}}"}[2m]))

---
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: latency
  namespace: adx-core-production
spec:
  args:
  - name: service-name
  
  metrics:
  - name: latency
    interval: 1m
    count: 5
    successCondition: result[0] <= 0.2
    failureLimit: 3
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket{service="{{args.service-name}}"}[2m])) by (le)
          )
```

### Build Pipeline
```yaml
# GitHub Actions workflow
name: Build and Deploy

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: adxcore/api

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: adx_core_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: |
        cargo test --all-features --workspace
        cargo clippy --all-targets --all-features -- -D warnings
        cargo fmt --all -- --check
      env:
        DATABASE_URL: postgres://postgres:postgres@localhost:5432/adx_core_test
        REDIS_URL: redis://localhost:6379
    
    - name: Security audit
      run: |
        cargo install cargo-audit
        cargo audit
    
    - name: Coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out xml --output-dir coverage
    
    - name: Upload coverage
      uses: codecov/codecov-action@v3
      with:
        file: coverage/cobertura.xml

  build:
    needs: test
    runs-on: ubuntu-latest
    
    outputs:
      image-digest: ${{ steps.build.outputs.digest }}
      image-tag: ${{ steps.meta.outputs.tags }}
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Docker Buildx
      uses: docker/setup-buildx-action@v2
    
    - name: Login to Container Registry
      uses: docker/login-action@v2
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v4
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=sha,prefix={{branch}}-
          type=raw,value=latest,enable={{is_default_branch}}
    
    - name: Build and push
      id: build
      uses: docker/build-push-action@v4
      with:
        context: .
        platforms: linux/amd64,linux/arm64
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max
        build-args: |
          BUILDKIT_INLINE_CACHE=1

  security-scan:
    needs: build
    runs-on: ubuntu-latest
    
    steps:
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        image-ref: ${{ needs.build.outputs.image-tag }}
        format: 'sarif'
        output: 'trivy-results.sarif'
    
    - name: Upload Trivy scan results
      uses: github/codeql-action/upload-sarif@v2
      with:
        sarif_file: 'trivy-results.sarif'

  deploy-staging:
    needs: [build, security-scan]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/develop'
    
    environment:
      name: staging
      url: https://staging-api.adxcore.com
    
    steps:
    - name: Deploy to staging
      run: |
        # Update ArgoCD application with new image
        curl -X PATCH \
          -H "Authorization: Bearer ${{ secrets.ARGOCD_TOKEN }}" \
          -H "Content-Type: application/json" \
          -d '{
            "spec": {
              "source": {
                "kustomize": {
                  "images": ["${{ needs.build.outputs.image-tag }}"]
                }
              }
            }
          }' \
          https://argocd.adxcore.com/api/v1/applications/adx-core-staging

  deploy-production:
    needs: [build, security-scan]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    environment:
      name: production
      url: https://api.adxcore.com
    
    steps:
    - name: Deploy to production
      run: |
        # Update ArgoCD application with new image
        curl -X PATCH \
          -H "Authorization: Bearer ${{ secrets.ARGOCD_TOKEN }}" \
          -H "Content-Type: application/json" \
          -d '{
            "spec": {
              "source": {
                "kustomize": {
                  "images": ["${{ needs.build.outputs.image-tag }}"]
                }
              }
            }
          }' \
          https://argocd.adxcore.com/api/v1/applications/adx-core-production
    
    - name: Notify deployment
      uses: 8398a7/action-slack@v3
      with:
        status: ${{ job.status }}
        channel: '#deployments'
        webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

## Disaster Recovery and Business Continuity

### Multi-Region Failover
```rust
// Disaster recovery orchestrator
pub struct DisasterRecoveryOrchestrator {
    health_monitor: Arc<HealthMonitor>,
    failover_manager: Arc<FailoverManager>,
    data_replicator: Arc<DataReplicator>,
    dns_manager: Arc<DNSManager>,
    notification_service: Arc<NotificationService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverPlan {
    pub plan_id: String,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub failover_steps: Vec<FailoverStep>,
    pub rollback_steps: Vec<RollbackStep>,
    pub estimated_rto: Duration, // Recovery Time Objective
    pub estimated_rpo: Duration, // Recovery Point Objective
    pub validation_checks: Vec<ValidationCheck>,
}

impl DisasterRecoveryOrchestrator {
    pub async fn monitor_and_failover(&self) -> Result<(), DRError> {
        loop {
            // Monitor primary region health
            let health_status = self.health_monitor.check_primary_region().await?;
            
            if health_status.requires_failover() {
                info!("Initiating disaster recovery failover");
                
                // Execute failover plan
                let failover_result = self.execute_failover(&health_status).await?;
                
                // Validate failover success
                self.validate_failover(&failover_result).await?;
                
                // Notify stakeholders
                self.notify_failover_completion(&failover_result).await?;
                
                // Monitor for recovery
                self.monitor_for_recovery().await?;
            }
            
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
    
    async fn execute_failover(&self, health_status: &HealthStatus) -> Result<FailoverResult, DRError> {
        let failover_plan = self.get_failover_plan(&health_status.failure_type).await?;
        
        let mut results = Vec::new();
        
        for step in &failover_plan.failover_steps {
            let step_result = match step {
                FailoverStep::StopTrafficToPrimary => {
                    self.dns_manager.redirect_traffic_to_secondary().await?
                }
                FailoverStep::PromoteSecondaryDatabase => {
                    self.data_replicator.promote_secondary_to_primary().await?
                }
                FailoverStep::StartSecondaryServices => {
                    self.failover_manager.start_secondary_region_services().await?
                }
                FailoverStep::ValidateSecondaryHealth => {
                    self.health_monitor.validate_secondary_region().await?
                }
                FailoverStep::UpdateLoadBalancer => {
                    self.dns_manager.update_load_balancer_config().await?
                }
            };
            
            results.push(step_result);
        }
        
        Ok(FailoverResult {
            plan_id: failover_plan.plan_id,
            executed_steps: results,
            completion_time: Utc::now(),
            success: true,
        })
    }
    
    pub async fn execute_recovery(&self, recovery_plan: &RecoveryPlan) -> Result<RecoveryResult, DRError> {
        info!("Starting disaster recovery process");
        
        // Step 1: Assess primary region status
        let primary_status = self.health_monitor.assess_primary_region().await?;
        
        if !primary_status.is_recoverable() {
            return Err(DRError::PrimaryRegionNotRecoverable);
        }
        
        // Step 2: Prepare primary region for recovery
        self.prepare_primary_for_recovery(&primary_status).await?;
        
        // Step 3: Sync data from secondary to primary
        let sync_result = self.data_replicator.sync_secondary_to_primary().await?;
        
        // Step 4: Validate data consistency
        self.validate_data_consistency(&sync_result).await?;
        
        // Step 5: Gradually shift traffic back to primary
        self.gradual_traffic_shift_to_primary().await?;
        
        // Step 6: Validate primary region performance
        let performance_validation = self.validate_primary_performance().await?;
        
        // Step 7: Complete recovery
        self.complete_recovery(&performance_validation).await?;
        
        Ok(RecoveryResult {
            recovery_id: Uuid::new_v4().to_string(),
            started_at: recovery_plan.started_at,
            completed_at: Utc::now(),
            data_loss: sync_result.data_loss_assessment,
            performance_impact: performance_validation.impact_assessment,
        })
    }
}
```

### Backup and Restore Strategy
```yaml
# Velero backup configuration
apiVersion: velero.io/v1
kind: Schedule
metadata:
  name: adx-core-daily-backup
  namespace: velero
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  template:
    includedNamespaces:
    - adx-core-production
    - monitoring
    
    excludedResources:
    - events
    - events.events.k8s.io
    
    storageLocation: default
    volumeSnapshotLocations:
    - default
    
    ttl: 720h  # 30 days
    
    hooks:
      resources:
      - name: postgres-backup-hook
        includedNamespaces:
        - adx-core-production
        includedResources:
        - pods
        labelSelector:
          matchLabels:
            app: postgres
        pre:
        - exec:
            container: postgres
            command:
            - /bin/bash
            - -c
            - "pg_dump -U $POSTGRES_USER $POSTGRES_DB > /tmp/backup.sql"
        post:
        - exec:
            container: postgres
            command:
            - /bin/bash
            - -c
            - "rm -f /tmp/backup.sql"

---
# Cross-region backup replication
apiVersion: batch/v1
kind: CronJob
metadata:
  name: cross-region-backup-sync
  namespace: adx-core-production
spec:
  schedule: "0 4 * * *"  # Daily at 4 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup-sync
            image: amazon/aws-cli:latest
            command:
            - /bin/bash
            - -c
            - |
              # Sync backups to secondary region
              aws s3 sync s3://adx-core-backups-us-east-1/ s3://adx-core-backups-eu-west-1/ --delete
              
              # Sync to tertiary region
              aws s3 sync s3://adx-core-backups-us-east-1/ s3://adx-core-backups-ap-south-1/ --delete
              
              # Verify backup integrity
              aws s3api head-object --bucket adx-core-backups-eu-west-1 --key latest/backup.tar.gz
              aws s3api head-object --bucket adx-core-backups-ap-south-1 --key latest/backup.tar.gz
            
            env:
            - name: AWS_ACCESS_KEY_ID
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: access-key-id
            - name: AWS_SECRET_ACCESS_KEY
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: secret-access-key
          
          restartPolicy: OnFailure
```

This comprehensive infrastructure and deployment architecture provides:

1. **Multi-cloud, multi-region architecture** with automatic failover
2. **Kubernetes-native deployment** with service mesh and security
3. **Advanced monitoring and observability** with Prometheus, Grafana, and Jaeger
4. **GitOps-based CI/CD** with progressive delivery and automated rollbacks
5. **Comprehensive security** with network policies, RBAC, and container scanning
6. **Disaster recovery** with automated failover and data replication
7. **High availability database** with clustering and backup strategies
8. **Performance optimization** with auto-scaling and resource management
9. **Enterprise-grade monitoring** with SLA tracking and alerting
10. **Business continuity** with comprehensive backup and restore procedures

The architecture is designed to support enterprise-scale deployment with 99.9% uptime SLA and comprehensive disaster recovery capabilities.