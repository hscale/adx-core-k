# ADX Core AI Service

The AI Service provides Temporal-first AI workflow orchestration with support for multiple AI providers, comprehensive usage tracking, and enterprise-grade monitoring.

## Features

### Core Capabilities
- **Multi-Provider Support**: OpenAI, Anthropic, and Local AI models
- **Temporal-First Architecture**: All complex AI operations as workflows
- **Usage Tracking**: Comprehensive token and cost monitoring
- **Health Monitoring**: Real-time provider health checks
- **Quota Management**: Per-tenant usage limits and enforcement
- **Content Moderation**: Built-in content filtering and safety

### AI Capabilities
- **Text Generation**: Creative writing, completion, and generation
- **Text Classification**: Category assignment and content analysis
- **Text Summarization**: Extractive and abstractive summarization
- **Entity Extraction**: Named entity recognition and extraction
- **Sentiment Analysis**: Emotion and sentiment detection

### Workflow Integration
- **User Onboarding**: AI-enhanced personalized onboarding
- **Document Processing**: Automated document analysis and extraction
- **Email Generation**: Intelligent email composition and optimization

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Service Architecture                  │
├─────────────────────────────────────────────────────────────┤
│  HTTP Server (Port 8086)     │  Temporal Worker            │
│  ├─── Direct Endpoints       │  ├─── AI Activities         │
│  ├─── Health Checks          │  ├─── AI Workflows          │
│  └─── Usage Analytics        │  └─── Provider Management   │
├─────────────────────────────────────────────────────────────┤
│                    AI Provider Layer                       │
│  ├─── OpenAI Provider        │  ├─── Anthropic Provider    │
│  ├─── Local AI Provider      │  └─── Provider Manager      │
├─────────────────────────────────────────────────────────────┤
│                    Data & Monitoring Layer                 │
│  ├─── Usage Tracker          │  ├─── Health Monitor        │
│  ├─── Model Registry         │  └─── Cost Calculator       │
├─────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                    │
│  ├─── PostgreSQL             │  ├─── Redis Cache           │
│  ├─── Temporal Server        │  └─── Metrics & Logging     │
└─────────────────────────────────────────────────────────────┘
```

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/adx_core
REDIS_URL=redis://localhost:6379

# Temporal
TEMPORAL_SERVER_URL=http://localhost:7233

# AI Providers
OPENAI_API_KEY=your_openai_api_key
ANTHROPIC_API_KEY=your_anthropic_api_key

# Local AI (optional)
AI_SERVICE_AI_PROVIDERS__LOCAL__ENABLED=true
AI_SERVICE_AI_PROVIDERS__LOCAL__BASE_URL=http://localhost:11434

# Monitoring
AI_SERVICE_MONITORING__METRICS_ENABLED=true
AI_SERVICE_MONITORING__USAGE_TRACKING_ENABLED=true
AI_SERVICE_MONITORING__COST_TRACKING_ENABLED=true

# Security
AI_SERVICE_SECURITY__JWT_SECRET=your_jwt_secret
AI_SERVICE_SECURITY__RATE_LIMIT_PER_MINUTE=60
```

## Usage

### Starting the Service

```bash
# HTTP Server Mode
cargo run --bin ai-service server --port 8086

# Temporal Worker Mode
cargo run --bin ai-service worker --task-queue ai-task-queue

# Using environment variable mode selection
AI_SERVICE_MODE=server cargo run --bin ai-service
AI_SERVICE_MODE=worker cargo run --bin ai-service
```

### API Endpoints

#### Health and Status
```bash
# Service health
GET /health

# Provider health
GET /health/providers/openai
GET /health/providers/anthropic
GET /health/providers/local

# Health history
GET /health/providers/openai/history?hours=24

# Availability metrics
GET /health/providers/openai/metrics?hours=24

# Alert conditions
GET /health/alerts
```

#### AI Operations
```bash
# Get available models
GET /api/v1/models
GET /api/v1/models/capability?capability=TextGeneration

# Generate text
POST /api/v1/generate
{
  "prompt": "Write a professional email about...",
  "model": "gpt-3.5-turbo",
  "parameters": {
    "max_tokens": 500,
    "temperature": 0.7
  }
}

# Classify text
POST /api/v1/classify
{
  "text": "This is a customer complaint about...",
  "categories": ["complaint", "inquiry", "compliment", "urgent"],
  "model": "gpt-3.5-turbo"
}

# Summarize text
POST /api/v1/summarize
{
  "text": "Long document content...",
  "max_length": 200,
  "style": "Executive",
  "model": "gpt-4"
}

# Extract entities
POST /api/v1/extract-entities
{
  "text": "John Doe from Acme Corp called about the contract...",
  "entity_types": ["Person", "Organization", "Date", "Money"],
  "model": "gpt-3.5-turbo"
}
```

#### Usage and Analytics
```bash
# Usage statistics
GET /api/v1/usage/stats?period_start=2024-01-01T00:00:00Z&period_end=2024-01-31T23:59:59Z

# Cost breakdown
GET /api/v1/usage/costs?period_start=2024-01-01T00:00:00Z&period_end=2024-01-31T23:59:59Z
```

### Temporal Workflows

#### User Onboarding Workflow
```rust
use ai_service::workflows::UserOnboardingAIRequest;

let request = UserOnboardingAIRequest {
    user_id: "user123".to_string(),
    tenant_id: "tenant456".to_string(),
    user_profile: UserProfile {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        role: "Developer".to_string(),
        department: Some("Engineering".to_string()),
        experience_level: ExperienceLevel::Intermediate,
        interests: vec!["API Development".to_string(), "AI/ML".to_string()],
    },
    onboarding_preferences: OnboardingPreferences {
        communication_style: CommunicationStyle::Technical,
        learning_pace: LearningPace::Fast,
        preferred_features: vec!["API Gateway".to_string(), "Workflows".to_string()],
        goals: vec!["Build APIs".to_string(), "Automate processes".to_string()],
    },
};

// Execute workflow
let result = temporal_client.execute_workflow(
    "user_onboarding_ai_workflow",
    request,
    WorkflowOptions::default(),
).await?;
```

#### Document Processing Workflow
```rust
use ai_service::workflows::DocumentProcessingAIRequest;

let request = DocumentProcessingAIRequest {
    document_id: "doc123".to_string(),
    tenant_id: "tenant456".to_string(),
    user_id: "user123".to_string(),
    document_content: "Contract content...".to_string(),
    document_type: DocumentType::Contract,
    processing_options: DocumentProcessingOptions {
        extract_entities: true,
        generate_summary: true,
        classify_content: true,
        extract_key_points: true,
        sentiment_analysis: false,
        custom_instructions: None,
    },
};

let result = temporal_client.execute_workflow(
    "document_processing_ai_workflow",
    request,
    WorkflowOptions::default(),
).await?;
```

#### Email Generation Workflow
```rust
use ai_service::workflows::EmailGenerationAIRequest;

let request = EmailGenerationAIRequest {
    tenant_id: "tenant456".to_string(),
    user_id: "user123".to_string(),
    email_type: EmailType::FollowUp,
    recipient_info: RecipientInfo {
        name: Some("Jane Smith".to_string()),
        role: Some("Product Manager".to_string()),
        company: Some("Acme Corp".to_string()),
        relationship: Some("Client".to_string()),
        communication_style: Some(CommunicationStyle::Professional),
    },
    email_context: EmailContext {
        subject_hint: Some("Project Update".to_string()),
        key_points: vec![
            "Project is on track".to_string(),
            "Next milestone in 2 weeks".to_string(),
            "Need feedback on requirements".to_string(),
        ],
        call_to_action: Some("Please review and provide feedback".to_string()),
        deadline: Some(chrono::Utc::now() + chrono::Duration::days(3)),
        reference_data: HashMap::new(),
    },
    generation_options: EmailGenerationOptions {
        tone: EmailTone::Professional,
        length: EmailLength::Medium,
        include_signature: true,
        personalization_level: PersonalizationLevel::High,
        language: None,
    },
};

let result = temporal_client.execute_workflow(
    "email_generation_ai_workflow",
    request,
    WorkflowOptions::default(),
).await?;
```

## Model Configuration

### Supported Models

#### OpenAI Models
- **gpt-3.5-turbo**: Fast, cost-effective for most tasks
- **gpt-4**: High-quality reasoning and complex tasks
- **gpt-4-turbo**: Latest model with extended context

#### Anthropic Models
- **claude-3-haiku**: Fast and efficient
- **claude-3-sonnet**: Balanced performance and cost
- **claude-3-opus**: Highest capability (when available)

#### Local Models
- **llama2-7b**: Open-source alternative
- **mistral-7b**: Code-focused model

### Model Selection Strategy

The service automatically selects the best model based on:
1. **Tenant subscription tier** (Free, Professional, Enterprise)
2. **Required capability** (TextGeneration, Classification, etc.)
3. **Cost optimization** (lowest cost per token for tier)
4. **Availability** (provider health status)

## Monitoring and Observability

### Health Monitoring
- **Provider Health**: Real-time status of AI providers
- **Model Availability**: Per-model health and response times
- **Error Tracking**: Automatic error detection and alerting
- **Performance Metrics**: Response times and success rates

### Usage Analytics
- **Token Tracking**: Detailed token usage by model and capability
- **Cost Analysis**: Real-time cost tracking and projections
- **Quota Monitoring**: Usage limits and threshold alerts
- **Trend Analysis**: Historical usage patterns and forecasting

### Alerting
- **Provider Outages**: Immediate notification of provider issues
- **High Response Times**: Performance degradation alerts
- **Quota Violations**: Usage limit breach notifications
- **Cost Thresholds**: Budget overrun warnings

## Security and Compliance

### Content Moderation
- **Input Filtering**: Automatic content safety checks
- **Output Validation**: Response content moderation
- **Policy Enforcement**: Configurable content policies
- **Audit Logging**: Complete content moderation history

### Data Protection
- **Encryption**: All data encrypted in transit and at rest
- **Access Control**: Role-based access to AI capabilities
- **Audit Trails**: Complete usage and access logging
- **Data Retention**: Configurable data retention policies

### Quota Management
- **Per-Tenant Limits**: Customizable usage quotas
- **Real-time Enforcement**: Immediate quota violation prevention
- **Grace Periods**: Configurable overrun allowances
- **Automatic Scaling**: Dynamic quota adjustments

## Development

### Running Tests
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration_tests

# Workflow tests
cargo test --test workflow_tests

# All tests
cargo test
```

### Database Migrations
```bash
# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add new_migration_name

# Revert migration
sqlx migrate revert
```

### Local Development Setup
```bash
# Start dependencies
docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d

# Run database migrations
sqlx migrate run

# Start AI service in development mode
cargo run --bin ai-service server

# Start worker in separate terminal
cargo run --bin ai-service worker
```

## Deployment

### Docker Deployment
```dockerfile
# Build image
docker build -t adx-core/ai-service .

# Run server
docker run -p 8086:8086 \
  -e DATABASE_URL=postgresql://... \
  -e OPENAI_API_KEY=... \
  adx-core/ai-service server

# Run worker
docker run \
  -e DATABASE_URL=postgresql://... \
  -e TEMPORAL_SERVER_URL=http://temporal:7233 \
  adx-core/ai-service worker
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ai-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ai-service
  template:
    metadata:
      labels:
        app: ai-service
    spec:
      containers:
      - name: ai-service
        image: adx-core/ai-service:latest
        ports:
        - containerPort: 8086
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: ai-service-secrets
              key: database-url
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: ai-service-secrets
              key: openai-api-key
```

## Troubleshooting

### Common Issues

#### Provider Connection Issues
```bash
# Check provider health
curl http://localhost:8086/health/providers/openai

# View provider logs
docker logs ai-service | grep -i "provider"
```

#### High Response Times
```bash
# Check model metrics
curl http://localhost:8086/health/providers/openai/metrics?hours=1

# Monitor resource usage
docker stats ai-service
```

#### Quota Exceeded Errors
```bash
# Check current usage
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8086/api/v1/usage/stats

# View quota configuration
psql -d adx_core -c "SELECT * FROM ai_quotas WHERE tenant_id = 'your-tenant-id';"
```

### Performance Tuning

#### Database Optimization
```sql
-- Analyze query performance
EXPLAIN ANALYZE SELECT * FROM ai_usage_records WHERE tenant_id = 'tenant123';

-- Update table statistics
ANALYZE ai_usage_records;

-- Vacuum tables
VACUUM ANALYZE ai_usage_records;
```

#### Redis Optimization
```bash
# Monitor Redis memory usage
redis-cli info memory

# Check key expiration
redis-cli ttl usage:tenant123:TextGeneration:hour:2024010112
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Run clippy for linting (`cargo clippy`)
- Add documentation for public APIs
- Include integration tests for workflows

## License

This project is licensed under the MIT License - see the LICENSE file for details.