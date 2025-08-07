# Workflow Service - Technical Design

## Architecture Overview

The Workflow Service leverages Temporal.io's proven workflow orchestration capabilities while adding a simple AI enhancement layer. This design prioritizes reliability and simplicity over complex AI orchestration.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Workflow Service                             │
├─────────────────┬─────────────────┬─────────────────────────────┤
│  Temporal.io    │   AI Service    │    Workflow Templates       │
│  Integration    │   Integration   │       & Registry            │
│                 │                 │                             │
│ • Workflow Def  │ • Simple AI     │ • Standard Templates        │
│ • Activity Def  │   Activities    │ • Custom Workflows          │
│ • Worker Mgmt   │ • Model Select  │ • Template Versioning       │
│ • Error Handle  │ • Fallback      │ • Workflow Sharing          │
└─────────────────┴─────────────────┴─────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Temporal    │    │   AI Service  │    │   PostgreSQL  │
│   Cluster     │    │   Providers   │    │   (Metadata)  │
└───────────────┘    └───────────────┘    └───────────────┘
```

## Core Components

### 1. Temporal.io Integration

**Workflow Engine**
- Uses Temporal.io for all workflow orchestration
- Leverages Temporal's durability and error handling
- Implements workflow versioning and migration
- Provides workflow history and replay capabilities

**Worker Management**
- Configurable worker pools for different workflow types
- Auto-scaling workers based on queue depth
- Worker health monitoring and restart
- Resource isolation for AI-intensive workflows

**Activity Framework**
- Standard activities for common operations
- AI-enhanced activities for premium features
- Activity retry policies and error handling
- Activity result caching for performance

### 2. AI Service Integration

**Simple AI Service**
```rust
pub struct AIService {
    providers: HashMap<String, Box<dyn AIProvider>>,
    model_selector: ModelSelector,
    fallback_enabled: bool,
    cost_tracker: CostTracker,
}

pub trait AIProvider {
    async fn generate_text(&self, prompt: &str) -> Result<AIResponse>;
    async fn classify(&self, text: &str, categories: &[String]) -> Result<String>;
    async fn summarize(&self, text: &str, max_length: usize) -> Result<String>;
    async fn extract_entities(&self, text: &str) -> Result<Vec<Entity>>;
}
```

**Model Selection Strategy**
- Tier-based model selection (Basic: none, Premium: GPT-3.5, Enterprise: GPT-4)
- Cost-based optimization within tier limits
- Automatic fallback to cheaper models on quota limits
- Provider failover for reliability

**AI Activities**
- `ai_generate_text_activity` - General text generation
- `ai_classify_content_activity` - Content classification
- `ai_summarize_activity` - Text summarization
- `ai_extract_entities_activity` - Named entity extraction
- `ai_personalize_content_activity` - Content personalization
- `ai_optimize_timing_activity` - Optimal timing suggestions

### 3. Workflow Templates

**Standard Templates**
- User onboarding workflow
- Document processing workflow
- Email campaign workflow
- Data synchronization workflow
- Backup and cleanup workflow
- Notification workflow

**Template Structure**
```rust
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: WorkflowCategory,
    pub version: String,
    pub parameters: Vec<WorkflowParameter>,
    pub ai_enhancement_points: Vec<AIEnhancementPoint>,
    pub workflow_definition: WorkflowDefinition,
}
```

**AI Enhancement Points**
- Configurable points where AI can add value
- Tier-based availability (Premium/Enterprise only)
- Fallback behavior when AI unavailable
- Performance impact tracking

### 4. Workflow Execution Engine

**Hybrid Execution Pattern**
```rust
#[workflow]
pub async fn hybrid_workflow(
    input: WorkflowInput,
    ai_enhanced: bool,
) -> WorkflowResult<WorkflowOutput> {
    // Step 1: Standard processing (always executed)
    let standard_result = standard_processing_activity(input.clone()).await?;
    
    // Step 2: AI enhancement (if enabled and available)
    let enhanced_result = if ai_enhanced {
        match ai_enhancement_activity(standard_result.clone()).await {
            Ok(enhanced) => enhanced,
            Err(_) => {
                // Fallback to standard result
                warn!("AI enhancement failed, using standard result");
                standard_result
            }
        }
    } else {
        standard_result
    };
    
    // Step 3: Final processing
    let final_result = finalize_processing_activity(enhanced_result).await?;
    
    Ok(WorkflowOutput {
        result: final_result,
        ai_enhanced,
        execution_metadata: get_execution_metadata(),
    })
}
```

## Database Schema

### Workflow Metadata
```sql
CREATE TABLE workflow_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    version VARCHAR(20) NOT NULL,
    definition JSONB NOT NULL,
    ai_enhancement_points JSONB DEFAULT '[]',
    tenant_id UUID,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(name, version, tenant_id)
);

CREATE TABLE workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_definition_id UUID NOT NULL REFERENCES workflow_definitions(id),
    temporal_workflow_id VARCHAR(255) NOT NULL,
    temporal_run_id VARCHAR(255) NOT NULL,
    
    -- Execution details
    status VARCHAR(20) NOT NULL DEFAULT 'running',
    input JSONB,
    output JSONB,
    error_message TEXT,
    
    -- AI enhancement tracking
    ai_enhanced BOOLEAN NOT NULL DEFAULT FALSE,
    ai_activities_used JSONB DEFAULT '[]',
    ai_cost_cents INTEGER DEFAULT 0,
    fallback_used BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Performance metrics
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- Metadata
    tenant_id UUID NOT NULL,
    triggered_by UUID,
    metadata JSONB DEFAULT '{}',
    
    UNIQUE(temporal_workflow_id, temporal_run_id)
);
```

### AI Usage Tracking
```sql
CREATE TABLE ai_usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_execution_id UUID REFERENCES workflow_executions(id),
    activity_name VARCHAR(100) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    model VARCHAR(100) NOT NULL,
    
    -- Usage metrics
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost_cents INTEGER,
    response_time_ms INTEGER,
    
    -- Quality metrics
    success BOOLEAN NOT NULL,
    confidence_score DECIMAL(3,2),
    fallback_used BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Audit
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    INDEX idx_ai_usage_tenant_date (tenant_id, created_at),
    INDEX idx_ai_usage_workflow (workflow_execution_id)
);
```

## API Endpoints

### Workflow Management
- `GET /workflows/templates` - List available workflow templates
- `POST /workflows/execute` - Execute a workflow
- `GET /workflows/executions` - List workflow executions
- `GET /workflows/executions/{id}` - Get execution details
- `POST /workflows/executions/{id}/cancel` - Cancel running workflow

### AI Enhancement
- `GET /workflows/ai/capabilities` - List available AI capabilities
- `POST /workflows/ai/test` - Test AI activity
- `GET /workflows/ai/usage` - Get AI usage statistics
- `GET /workflows/ai/costs` - Get AI cost breakdown

### Monitoring
- `GET /workflows/health` - Service health check
- `GET /workflows/metrics` - Workflow execution metrics
- `GET /workflows/performance` - Performance analytics

## Workflow Examples

### 1. User Onboarding Workflow
```rust
#[workflow]
pub async fn user_onboarding_workflow(
    user_data: UserRegistrationData,
    ai_enhanced: bool,
) -> WorkflowResult<OnboardingResult> {
    // Step 1: Create user account
    let user = create_user_activity(user_data.clone()).await?;
    
    // Step 2: Generate welcome message (AI-enhanced if available)
    let welcome_message = if ai_enhanced {
        ai_generate_personalized_welcome_activity(user.clone()).await
            .unwrap_or_else(|_| get_default_welcome_message())
    } else {
        get_default_welcome_message()
    };
    
    // Step 3: Send welcome email
    send_welcome_email_activity(user.email.clone(), welcome_message).await?;
    
    // Step 4: Generate setup tasks (AI-enhanced if available)
    let setup_tasks = if ai_enhanced {
        ai_generate_setup_tasks_activity(user.clone()).await
            .unwrap_or_else(|_| get_default_setup_tasks())
    } else {
        get_default_setup_tasks()
    };
    
    // Step 5: Create setup tasks
    create_setup_tasks_activity(user.id, setup_tasks.clone()).await?;
    
    Ok(OnboardingResult {
        user_id: user.id,
        setup_tasks_count: setup_tasks.len(),
        ai_enhanced,
        welcome_message_personalized: ai_enhanced,
    })
}
```

### 2. Document Processing Workflow
```rust
#[workflow]
pub async fn document_processing_workflow(
    document: Document,
    ai_enhanced: bool,
) -> WorkflowResult<ProcessedDocument> {
    // Step 1: Extract text
    let text = extract_text_activity(document.clone()).await?;
    
    // Step 2: AI processing (if enabled)
    let ai_results = if ai_enhanced {
        // Run AI activities in parallel
        let (summary, entities, classification) = temporal_sdk::join!(
            ai_summarize_activity(text.clone(), 200),
            ai_extract_entities_activity(text.clone()),
            ai_classify_content_activity(text.clone(), vec![
                "Contract".to_string(),
                "Invoice".to_string(),
                "Report".to_string(),
                "Other".to_string()
            ])
        );
        
        Some(AIProcessingResults {
            summary: summary.ok(),
            entities: entities.ok(),
            classification: classification.ok(),
        })
    } else {
        None
    };
    
    // Step 3: Store processed document
    let result = store_processed_document_activity(
        document.id,
        text,
        ai_results,
    ).await?;
    
    Ok(result)
}
```

## Performance Optimizations

### Caching Strategy
- AI response caching for repeated queries
- Workflow definition caching
- Template metadata caching
- Model configuration caching

### Resource Management
- Separate worker pools for AI and standard activities
- Resource limits for AI activities
- Queue prioritization for premium users
- Auto-scaling based on queue depth

### Cost Optimization
- Token usage tracking and limits
- Model selection based on cost/performance
- Batch processing for similar AI requests
- Caching to reduce redundant AI calls

## Monitoring and Observability

### Metrics
- Workflow execution rates and success rates
- AI activity usage and performance
- Cost tracking per tenant and activity
- Error rates and failure patterns
- Resource utilization and scaling metrics

### Logging
- Workflow execution logs with correlation IDs
- AI activity requests and responses
- Performance metrics and timing
- Error details and stack traces
- Cost and usage tracking

### Alerts
- Workflow failure rate thresholds
- AI service availability issues
- Cost budget overruns
- Performance degradation
- Resource exhaustion warnings

## Testing Strategy

### Unit Tests
- Individual activity testing
- AI service integration testing
- Workflow logic validation
- Error handling and fallback testing
- Cost calculation accuracy

### Integration Tests
- End-to-end workflow execution
- Temporal integration testing
- AI provider integration testing
- Database operations testing
- Performance benchmarking

### Load Tests
- Concurrent workflow execution
- AI activity performance under load
- Resource scaling validation
- Cost tracking accuracy
- Failure recovery testing

## Deployment Considerations

### Temporal Configuration
- Separate Temporal namespace per environment
- Worker configuration and scaling
- Workflow versioning and migration
- History retention policies

### AI Service Configuration
- API key management and rotation
- Rate limiting and quota management
- Provider failover configuration
- Cost monitoring and alerting

### Scaling Strategy
- Horizontal worker scaling
- AI activity resource isolation
- Database connection pooling
- Cache configuration and sizing