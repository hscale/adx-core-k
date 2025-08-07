# 📋 Specification-Based Development Rules

## Development Philosophy

> **"Code what's specified. Specify what's needed. Test what's built."**

These rules ensure every line of code traces back to a specification and every specification becomes working code.

## 🎯 Specification-Driven Development Process

### 1. Specification Analysis Workflow
```rust
// ✅ REQUIRED: Specification parsing template
#[derive(Debug, Clone)]
pub struct SpecificationRequirement {
    pub spec_id: String,
    pub spec_file: PathBuf,
    pub requirement_type: RequirementType,
    pub priority: Priority,
    pub acceptance_criteria: Vec<AcceptanceCriteria>,
    pub dependencies: Vec<String>,
    pub estimated_effort: EstimatedEffort,
}

#[derive(Debug, Clone)]
pub enum RequirementType {
    Functional,
    NonFunctional,
    Security,
    Performance,
    Integration,
    UserInterface,
}

#[derive(Debug, Clone)]
pub struct AcceptanceCriteria {
    pub given: String,      // Given this condition
    pub when: String,       // When this action occurs
    pub then: String,       // Then this result should happen
    pub test_method: TestMethod,
}

#[derive(Debug, Clone)]
pub enum TestMethod {
    UnitTest { test_file: String, test_function: String },
    IntegrationTest { test_scenario: String },
    ManualTest { test_steps: Vec<String> },
    PerformanceTest { metrics: Vec<String> },
}

// ✅ REQUIRED: Specification parser
pub async fn parse_specification_file(
    spec_file: &Path,
) -> Result<Vec<SpecificationRequirement>, SpecError> {
    let content = tokio::fs::read_to_string(spec_file).await?;
    
    // Parse markdown sections for requirements
    let mut requirements = Vec::new();
    let mut current_requirement = None;
    
    for line in content.lines() {
        match line.trim() {
            // Parse requirement headers
            line if line.starts_with("## ") => {
                if let Some(req) = current_requirement.take() {
                    requirements.push(req);
                }
                current_requirement = Some(parse_requirement_header(line)?);
            },
            
            // Parse acceptance criteria
            line if line.starts_with("- **Given") => {
                if let Some(ref mut req) = current_requirement {
                    let criteria = parse_acceptance_criteria(line)?;
                    req.acceptance_criteria.push(criteria);
                }
            },
            
            // Parse implementation code blocks
            line if line.starts_with("```rust") => {
                if let Some(ref mut req) = current_requirement {
                    let code_block = parse_code_block(&content, line)?;
                    req.implementation_hints.push(code_block);
                }
            },
            
            _ => {}
        }
    }
    
    if let Some(req) = current_requirement {
        requirements.push(req);
    }
    
    Ok(requirements)
}
```

### 2. Requirement Traceability
```rust
// ✅ REQUIRED: Requirement traceability system
#[derive(Debug, Clone)]
pub struct RequirementTrace {
    pub requirement_id: String,
    pub spec_file: PathBuf,
    pub implementation_files: Vec<PathBuf>,
    pub test_files: Vec<PathBuf>,
    pub documentation_files: Vec<PathBuf>,
    pub completion_status: CompletionStatus,
}

#[derive(Debug, Clone)]
pub enum CompletionStatus {
    NotStarted,
    InProgress { percentage: u8 },
    Implemented { tested: bool },
    Complete,
    Blocked { reason: String },
}

// ✅ REQUIRED: Generate traceability matrix
pub async fn generate_traceability_matrix(
    specs_dir: &Path,
    source_dir: &Path,
) -> Result<Vec<RequirementTrace>, TraceabilityError> {
    let mut traces = Vec::new();
    
    // 1. Parse all specification files
    let spec_files = find_spec_files(specs_dir).await?;
    
    for spec_file in spec_files {
        let requirements = parse_specification_file(&spec_file).await?;
        
        for requirement in requirements {
            let trace = RequirementTrace {
                requirement_id: requirement.spec_id.clone(),
                spec_file: spec_file.clone(),
                implementation_files: find_implementation_files(
                    source_dir, 
                    &requirement.spec_id
                ).await?,
                test_files: find_test_files(
                    source_dir, 
                    &requirement.spec_id
                ).await?,
                documentation_files: find_documentation_files(
                    source_dir, 
                    &requirement.spec_id
                ).await?,
                completion_status: assess_completion_status(&requirement).await?,
            };
            
            traces.push(trace);
        }
    }
    
    Ok(traces)
}

// ✅ REQUIRED: Implementation file finder
async fn find_implementation_files(
    source_dir: &Path,
    requirement_id: &str,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut implementation_files = Vec::new();
    
    // Search for requirement ID in comments
    let mut entries = tokio::fs::read_dir(source_dir).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.extension() == Some(std::ffi::OsStr::new("rs")) {
            let content = tokio::fs::read_to_string(&path).await?;
            
            // Look for requirement references in comments
            if content.contains(&format!("// REQ: {}", requirement_id)) ||
               content.contains(&format!("/// Implements requirement: {}", requirement_id)) {
                implementation_files.push(path);
            }
        }
    }
    
    Ok(implementation_files)
}
```

### 3. Code Generation from Specifications
```rust
// ✅ REQUIRED: Code generation template engine
pub struct CodeGenerator {
    templates: HashMap<String, Template>,
    context: GenerationContext,
}

#[derive(Debug, Clone)]
pub struct GenerationContext {
    pub service_name: String,
    pub module_name: String,
    pub requirements: Vec<SpecificationRequirement>,
    pub database_schema: DatabaseSchema,
    pub api_endpoints: Vec<ApiEndpoint>,
}

impl CodeGenerator {
    // ✅ Generate service structure from specifications
    pub async fn generate_service_from_spec(
        &self,
        spec_file: &Path,
    ) -> Result<GeneratedService, CodeGenError> {
        let requirements = parse_specification_file(spec_file).await?;
        
        let mut generated_files = Vec::new();
        
        // Generate main service file
        let main_file = self.generate_main_service(&requirements).await?;
        generated_files.push(main_file);
        
        // Generate workflow files
        for req in &requirements {
            if req.requirement_type == RequirementType::Functional {
                let workflow_file = self.generate_workflow_from_requirement(req).await?;
                generated_files.push(workflow_file);
            }
        }
        
        // Generate activity files
        for req in &requirements {
            for criteria in &req.acceptance_criteria {
                let activity_file = self.generate_activity_from_criteria(criteria).await?;
                generated_files.push(activity_file);
            }
        }
        
        // Generate test files
        let test_files = self.generate_tests_from_requirements(&requirements).await?;
        generated_files.extend(test_files);
        
        Ok(GeneratedService {
            service_name: self.context.service_name.clone(),
            files: generated_files,
            dependencies: extract_dependencies(&requirements),
            configuration: generate_configuration(&requirements),
        })
    }
    
    // ✅ Generate workflow from functional requirement
    async fn generate_workflow_from_requirement(
        &self,
        requirement: &SpecificationRequirement,
    ) -> Result<GeneratedFile, CodeGenError> {
        let template = self.templates.get("workflow").ok_or(
            CodeGenError::TemplateNotFound("workflow".to_string())
        )?;
        
        let workflow_name = format!("{}_workflow", 
            requirement.spec_id.to_lowercase().replace("-", "_")
        );
        
        let context = json!({
            "workflow_name": workflow_name,
            "requirement_id": requirement.spec_id,
            "description": extract_description(requirement),
            "input_type": generate_input_type(requirement),
            "output_type": generate_output_type(requirement),
            "activities": generate_activities_list(requirement),
            "error_handling": generate_error_handling(requirement),
        });
        
        let code = template.render(&context)?;
        
        Ok(GeneratedFile {
            path: PathBuf::from(format!("src/workflows/{}.rs", workflow_name)),
            content: code,
            file_type: FileType::Workflow,
        })
    }
    
    // ✅ Generate tests from acceptance criteria
    async fn generate_tests_from_requirements(
        &self,
        requirements: &[SpecificationRequirement],
    ) -> Result<Vec<GeneratedFile>, CodeGenError> {
        let mut test_files = Vec::new();
        
        for requirement in requirements {
            for criteria in &requirement.acceptance_criteria {
                let test_file = self.generate_test_from_criteria(requirement, criteria).await?;
                test_files.push(test_file);
            }
        }
        
        Ok(test_files)
    }
    
    async fn generate_test_from_criteria(
        &self,
        requirement: &SpecificationRequirement,
        criteria: &AcceptanceCriteria,
    ) -> Result<GeneratedFile, CodeGenError> {
        let template = match criteria.test_method {
            TestMethod::UnitTest { .. } => self.templates.get("unit_test"),
            TestMethod::IntegrationTest { .. } => self.templates.get("integration_test"),
            TestMethod::PerformanceTest { .. } => self.templates.get("performance_test"),
            _ => return Err(CodeGenError::UnsupportedTestMethod),
        }.ok_or(CodeGenError::TemplateNotFound("test".to_string()))?;
        
        let test_name = format!("test_{}_{}", 
            requirement.spec_id.to_lowercase().replace("-", "_"),
            criteria.when.to_lowercase().replace(" ", "_")
        );
        
        let context = json!({
            "test_name": test_name,
            "requirement_id": requirement.spec_id,
            "given": criteria.given,
            "when": criteria.when,
            "then": criteria.then,
            "test_setup": generate_test_setup(criteria),
            "test_execution": generate_test_execution(criteria),
            "test_assertions": generate_test_assertions(criteria),
        });
        
        let code = template.render(&context)?;
        
        Ok(GeneratedFile {
            path: PathBuf::from(format!("tests/{}.rs", test_name)),
            content: code,
            file_type: FileType::Test,
        })
    }
}
```

## 🧪 Specification Testing Framework

### 1. Behavior-Driven Development
```rust
// ✅ REQUIRED: BDD test framework for specifications
use cucumber::{given, when, then, World};

#[derive(Debug, Default, World)]
pub struct SpecificationWorld {
    pub context: TestContext,
    pub current_user: Option<User>,
    pub current_tenant: Option<Tenant>,
    pub api_response: Option<Response>,
    pub workflow_result: Option<WorkflowResult>,
}

#[derive(Debug, Default)]
pub struct TestContext {
    pub database: Option<TestDatabase>,
    pub temporal: Option<TestTemporalClient>,
    pub services: HashMap<String, TestService>,
}

// ✅ BDD steps for common scenarios
#[given("a user with email {string} exists in tenant {string}")]
async fn given_user_exists(
    world: &mut SpecificationWorld,
    email: String,
    tenant_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let tenant = world.context.database
        .as_ref()
        .unwrap()
        .create_test_tenant(&tenant_id)
        .await?;
    
    let user = world.context.database
        .as_ref()
        .unwrap()
        .create_test_user(&email, tenant.id)
        .await?;
    
    world.current_user = Some(user);
    world.current_tenant = Some(tenant);
    
    Ok(())
}

#[when("the user attempts to {string} resource {string}")]
async fn when_user_attempts_action(
    world: &mut SpecificationWorld,
    action: String,
    resource: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = world.current_user.as_ref().unwrap();
    let tenant = world.current_tenant.as_ref().unwrap();
    
    // Generate auth token
    let token = world.context.services
        .get("auth")
        .unwrap()
        .generate_token(user.id, tenant.id)
        .await?;
    
    // Make API request
    let response = world.context.services
        .get("api-gateway")
        .unwrap()
        .make_request(&HttpRequest {
            method: action_to_http_method(&action),
            path: resource_to_path(&resource),
            headers: vec![("Authorization", format!("Bearer {}", token))],
            body: None,
        })
        .await?;
    
    world.api_response = Some(response);
    
    Ok(())
}

#[then("the request should {string} with status {int}")]
async fn then_request_should_result(
    world: &mut SpecificationWorld,
    result: String,
    status_code: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = world.api_response.as_ref().unwrap();
    
    assert_eq!(response.status, status_code, 
        "Expected status {}, got {}", status_code, response.status);
    
    match result.as_str() {
        "succeed" => assert!(status_code >= 200 && status_code < 300),
        "fail" => assert!(status_code >= 400),
        _ => panic!("Unknown result expectation: {}", result),
    }
    
    Ok(())
}

#[then("a workflow {string} should be triggered")]
async fn then_workflow_should_be_triggered(
    world: &mut SpecificationWorld,
    workflow_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let temporal = world.context.temporal.as_ref().unwrap();
    
    // Check if workflow was started
    let workflows = temporal.list_workflows().await?;
    
    let matching_workflow = workflows.iter()
        .find(|w| w.workflow_type == workflow_name);
    
    assert!(matching_workflow.is_some(), 
        "Workflow {} was not triggered", workflow_name);
    
    Ok(())
}
```

### 2. Specification Compliance Testing
```rust
// ✅ REQUIRED: Automated specification compliance testing
pub struct SpecificationComplianceTest {
    pub spec_file: PathBuf,
    pub implementation_files: Vec<PathBuf>,
    pub test_results: Vec<ComplianceTestResult>,
}

#[derive(Debug, Clone)]
pub struct ComplianceTestResult {
    pub requirement_id: String,
    pub test_type: ComplianceTestType,
    pub status: TestStatus,
    pub details: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ComplianceTestType {
    RequirementImplemented,
    AcceptanceCriteriaMet,
    PerformanceRequirementMet,
    SecurityRequirementMet,
    ApiContractCompliant,
}

impl SpecificationComplianceTest {
    // ✅ Test if all requirements are implemented
    pub async fn test_requirements_implemented(&mut self) -> Result<(), TestError> {
        let requirements = parse_specification_file(&self.spec_file).await?;
        
        for requirement in requirements {
            let implementation_found = self.find_requirement_implementation(&requirement).await?;
            
            let result = ComplianceTestResult {
                requirement_id: requirement.spec_id.clone(),
                test_type: ComplianceTestType::RequirementImplemented,
                status: if implementation_found.is_some() {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                details: if let Some(impl_file) = implementation_found {
                    format!("Implemented in {}", impl_file.display())
                } else {
                    "No implementation found".to_string()
                },
                evidence: vec![],
            };
            
            self.test_results.push(result);
        }
        
        Ok(())
    }
    
    // ✅ Test if acceptance criteria are covered by tests
    pub async fn test_acceptance_criteria_coverage(&mut self) -> Result<(), TestError> {
        let requirements = parse_specification_file(&self.spec_file).await?;
        
        for requirement in requirements {
            for criteria in requirement.acceptance_criteria {
                let test_found = self.find_criteria_test(&criteria).await?;
                
                let result = ComplianceTestResult {
                    requirement_id: requirement.spec_id.clone(),
                    test_type: ComplianceTestType::AcceptanceCriteriaMet,
                    status: if test_found.is_some() {
                        TestStatus::Passed
                    } else {
                        TestStatus::Failed
                    },
                    details: format!("Criteria: {} -> {}", criteria.when, criteria.then),
                    evidence: test_found.map(|f| vec![f.display().to_string()]).unwrap_or_default(),
                };
                
                self.test_results.push(result);
            }
        }
        
        Ok(())
    }
    
    // ✅ Find implementation of specific requirement
    async fn find_requirement_implementation(
        &self,
        requirement: &SpecificationRequirement,
    ) -> Result<Option<PathBuf>, std::io::Error> {
        for impl_file in &self.implementation_files {
            let content = tokio::fs::read_to_string(impl_file).await?;
            
            // Look for requirement reference
            if content.contains(&format!("// REQ: {}", requirement.spec_id)) ||
               content.contains(&format!("/// Implements requirement: {}", requirement.spec_id)) {
                return Ok(Some(impl_file.clone()));
            }
            
            // Look for implementation patterns
            if self.matches_implementation_pattern(&content, requirement) {
                return Ok(Some(impl_file.clone()));
            }
        }
        
        Ok(None)
    }
    
    fn matches_implementation_pattern(
        &self,
        content: &str,
        requirement: &SpecificationRequirement,
    ) -> bool {
        // Check for workflow implementations
        if requirement.requirement_type == RequirementType::Functional {
            let workflow_name = format!("{}_workflow", 
                requirement.spec_id.to_lowercase().replace("-", "_")
            );
            
            if content.contains(&workflow_name) {
                return true;
            }
        }
        
        // Check for API endpoint implementations
        if let Some(endpoint) = extract_api_endpoint(requirement) {
            if content.contains(&endpoint.path) && 
               content.contains(&endpoint.method.to_string()) {
                return true;
            }
        }
        
        false
    }
}
```

## 📊 Specification Quality Metrics

### 1. Specification Coverage Analysis
```rust
// ✅ REQUIRED: Specification coverage metrics
#[derive(Debug, Clone)]
pub struct SpecificationCoverage {
    pub total_requirements: usize,
    pub implemented_requirements: usize,
    pub tested_requirements: usize,
    pub documented_requirements: usize,
    pub coverage_percentage: f64,
    pub quality_score: f64,
}

impl SpecificationCoverage {
    pub async fn analyze_project_coverage(
        specs_dir: &Path,
        source_dir: &Path,
        tests_dir: &Path,
    ) -> Result<Self, CoverageError> {
        let traceability_matrix = generate_traceability_matrix(specs_dir, source_dir).await?;
        
        let total_requirements = traceability_matrix.len();
        let implemented_requirements = traceability_matrix.iter()
            .filter(|trace| !trace.implementation_files.is_empty())
            .count();
        let tested_requirements = traceability_matrix.iter()
            .filter(|trace| !trace.test_files.is_empty())
            .count();
        let documented_requirements = traceability_matrix.iter()
            .filter(|trace| !trace.documentation_files.is_empty())
            .count();
        
        let coverage_percentage = if total_requirements > 0 {
            (implemented_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };
        
        let quality_score = calculate_quality_score(
            implemented_requirements,
            tested_requirements,
            documented_requirements,
            total_requirements,
        );
        
        Ok(Self {
            total_requirements,
            implemented_requirements,
            tested_requirements,
            documented_requirements,
            coverage_percentage,
            quality_score,
        })
    }
}

fn calculate_quality_score(
    implemented: usize,
    tested: usize,
    documented: usize,
    total: usize,
) -> f64 {
    if total == 0 {
        return 0.0;
    }
    
    let implementation_score = (implemented as f64 / total as f64) * 40.0; // 40% weight
    let testing_score = (tested as f64 / total as f64) * 40.0;            // 40% weight
    let documentation_score = (documented as f64 / total as f64) * 20.0;   // 20% weight
    
    implementation_score + testing_score + documentation_score
}
```

### 2. Specification Quality Gates
```rust
// ✅ REQUIRED: Quality gates for specification compliance
pub struct SpecificationQualityGates {
    pub minimum_coverage_percentage: f64,
    pub minimum_quality_score: f64,
    pub required_test_types: Vec<RequiredTestType>,
    pub performance_requirements: PerformanceRequirements,
}

#[derive(Debug, Clone)]
pub enum RequiredTestType {
    UnitTest,
    IntegrationTest,
    PerformanceTest,
    SecurityTest,
}

impl Default for SpecificationQualityGates {
    fn default() -> Self {
        Self {
            minimum_coverage_percentage: 90.0,
            minimum_quality_score: 85.0,
            required_test_types: vec![
                RequiredTestType::UnitTest,
                RequiredTestType::IntegrationTest,
            ],
            performance_requirements: PerformanceRequirements::default(),
        }
    }
}

impl SpecificationQualityGates {
    pub async fn validate_project_quality(
        &self,
        specs_dir: &Path,
        source_dir: &Path,
        tests_dir: &Path,
    ) -> Result<QualityValidationResult, ValidationError> {
        let coverage = SpecificationCoverage::analyze_project_coverage(
            specs_dir, source_dir, tests_dir
        ).await?;
        
        let mut violations = Vec::new();
        
        // Check coverage percentage
        if coverage.coverage_percentage < self.minimum_coverage_percentage {
            violations.push(QualityViolation {
                violation_type: ViolationType::InsufficientCoverage,
                message: format!(
                    "Specification coverage is {:.1}%, minimum required is {:.1}%",
                    coverage.coverage_percentage,
                    self.minimum_coverage_percentage
                ),
                severity: Severity::High,
            });
        }
        
        // Check quality score
        if coverage.quality_score < self.minimum_quality_score {
            violations.push(QualityViolation {
                violation_type: ViolationType::LowQualityScore,
                message: format!(
                    "Quality score is {:.1}, minimum required is {:.1}",
                    coverage.quality_score,
                    self.minimum_quality_score
                ),
                severity: Severity::Medium,
            });
        }
        
        // Check required test types
        for required_test_type in &self.required_test_types {
            let test_coverage = self.check_test_type_coverage(
                tests_dir, 
                required_test_type
            ).await?;
            
            if test_coverage < 80.0 {
                violations.push(QualityViolation {
                    violation_type: ViolationType::MissingTestType,
                    message: format!(
                        "{:?} coverage is {:.1}%, minimum required is 80%",
                        required_test_type,
                        test_coverage
                    ),
                    severity: Severity::Medium,
                });
            }
        }
        
        Ok(QualityValidationResult {
            is_valid: violations.is_empty(),
            coverage,
            violations,
        })
    }
}
```

---

**Every specification becomes code. Every requirement becomes a test. Nothing is left to chance!** 📋
