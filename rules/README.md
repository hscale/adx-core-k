# 🤖 AI Coding Rules & Prompts System

This directory contains comprehensive AI coding prompts and rules for the ADX Core project. These prompts are designed to maintain consistency, quality, and architectural integrity across all AI-assisted development.

## 📁 Directory Structure

```
rules/
├── README.md                    # This file
├── core/                        # Core system rules
│   ├── architecture.md          # Temporal-first architecture rules
│   ├── coding-standards.md      # Rust coding standards
│   ├── security.md              # Security requirements
│   └── performance.md           # Performance guidelines
├── workflows/                   # Temporal workflow prompts
│   ├── workflow-patterns.md     # Common workflow patterns
│   ├── activity-design.md       # Activity implementation
│   └── error-handling.md        # Workflow error patterns
├── services/                    # Service-specific prompts
│   ├── api-gateway.md           # API Gateway development
│   ├── auth-service.md          # Authentication service
│   ├── file-service.md          # File management service
│   ├── workflow-service.md      # Workflow orchestration
│   └── shared-libraries.md     # Shared utilities
├── hooks/                       # Git hooks and automation
│   ├── pre-commit.md            # Pre-commit validations
│   ├── pre-push.md              # Pre-push checks
│   └── commit-message.md        # Commit message format
├── steering/                    # Development steering prompts
│   ├── phase-management.md      # Development phases
│   ├── team-handoff.md          # AI team transitions
│   └── priority-matrix.md       # Feature prioritization
└── specs/                       # Specification-based prompts
    ├── requirements.md          # Requirements analysis
    ├── design-patterns.md       # Design implementation
    └── testing.md               # Testing strategies
```

## 🎯 Usage Guidelines

### For AI Development Teams
1. **Start with Core Rules**: Always read `core/architecture.md` first
2. **Choose Your Service**: Select relevant service prompt from `services/`
3. **Follow Workflow Patterns**: Use `workflows/` for Temporal implementations
4. **Apply Quality Gates**: Use `hooks/` for validation
5. **Maintain Steering**: Reference `steering/` for project direction

### For Code Generation
1. **Input**: Combine relevant rule files with your specific task
2. **Context**: Include current project state from `PROJECT_TRACKING.md`
3. **Output**: Generate code following all applicable rules
4. **Validation**: Apply quality checks from `hooks/`

### For Architecture Decisions
1. **Reference**: Use `specs/` prompts for specification alignment
2. **Consistency**: Apply `core/architecture.md` patterns
3. **Quality**: Follow `core/coding-standards.md` guidelines
4. **Security**: Validate against `core/security.md` requirements

## 🚀 Quick Start Examples

### Generate a New Service
```bash
# Combine these prompts:
cat rules/core/architecture.md rules/services/template.md rules/workflows/workflow-patterns.md > combined_prompt.md
```

### Implement a Workflow
```bash
# Use these prompts:
cat rules/workflows/workflow-patterns.md rules/workflows/activity-design.md rules/core/performance.md > workflow_prompt.md
```

### Code Review
```bash
# Apply these checks:
cat rules/hooks/pre-commit.md rules/core/security.md rules/core/coding-standards.md > review_prompt.md
```

## 📊 Quality Metrics

Each prompt is designed to ensure:
- ✅ **Architectural Consistency**: Temporal-first patterns
- ✅ **Code Quality**: Rust best practices
- ✅ **Security**: Multi-tenant isolation
- ✅ **Performance**: Sub-100ms response times
- ✅ **Maintainability**: Clear documentation
- ✅ **Testability**: Comprehensive test coverage

## 🔄 Continuous Improvement

These prompts are living documents that evolve with the project:
- **Feedback Integration**: Update based on development experience
- **Pattern Recognition**: Extract successful patterns into reusable prompts
- **Quality Enhancement**: Refine prompts based on code review feedback
- **Team Learning**: Share successful prompt combinations

## 🎭 AI Team Persona

When using these prompts, embody the "Top 1% AI Coding Assistant" persona:
- **Expert Level**: Deep understanding of Rust, Temporal, and distributed systems
- **Pragmatic**: Balance perfection with practical delivery
- **Security-First**: Always consider multi-tenant security implications
- **Performance-Aware**: Optimize for scale and responsiveness
- **Documentation-Driven**: Clear, comprehensive documentation
- **Future-Proof**: Extensible, maintainable code

---

**Ready to code like the top 1%? Start with `core/architecture.md`!** 🚀
