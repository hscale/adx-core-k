---
name: AI Team Handoff
about: Document handoff between AI development teams
title: '[HANDOFF] AI Team Transition - [Date]'
labels: 'handoff, documentation'
assignees: ''
---

## 🔄 Team Transition Information

### Outgoing AI Team
- **Team ID**: [Previous team identifier]
- **Active Period**: [Start date] to [End date]
- **Last Commit**: [Commit hash/link]

### Incoming AI Team
- **Team ID**: [New team identifier]
- **Start Date**: [Today's date]
- **Assigned Phase**: [Phase 2/3/4 or specific module]

## ✅ Handoff Checklist

### Documentation Updates
- [ ] Updated `PROJECT_TRACKING.md` with current status
- [ ] Updated relevant specs in `.kiro/specs/`
- [ ] Documented any new issues or blockers
- [ ] Updated README.md if needed

### Code Status
- [ ] All changes committed and pushed
- [ ] No uncommitted work-in-progress
- [ ] Build status: [✅ Passing / ❌ Failing / 🟡 With warnings]
- [ ] Tests status: [✅ All passing / ❌ Some failing / 🟡 Skipped]

### Environment Verification
- [ ] `./scripts/dev-start.sh` works correctly
- [ ] All services start successfully
- [ ] Health checks pass: `curl http://localhost:8080/health`
- [ ] Database accessible and populated

## 📊 Current Status Summary

### Completed Work
List major accomplishments during this team's tenure:
- [ ] Task 1: [Description]
- [ ] Task 2: [Description]
- [ ] Task 3: [Description]

### In-Progress Work
Document any work that was started but not completed:
- [ ] **Task**: [Description]
  - **Status**: [% complete]
  - **Files**: [List modified files]
  - **Next Steps**: [What needs to be done]

### Blocked Items
List any items that are blocked and why:
- [ ] **Blocker**: [Description]
  - **Reason**: [Why it's blocked]
  - **Resolution**: [How to unblock]

## 🐛 Known Issues

### Critical Issues (🔴)
- [ ] **Issue**: [Description]
  - **Impact**: [How it affects development]
  - **Location**: [File/service affected]
  - **Workaround**: [If any]

### Non-Critical Issues (🟡)
- [ ] **Issue**: [Description]
  - **Priority**: [High/Medium/Low]
  - **Notes**: [Additional context]

## 🎯 Recommendations for Next Team

### Immediate Priorities (Next 1-2 days)
1. **Priority 1**: [Task description]
   - **Why**: [Reasoning]
   - **Resources**: [Relevant specs/docs]

2. **Priority 2**: [Task description]
   - **Why**: [Reasoning]
   - **Resources**: [Relevant specs/docs]

### Medium-term Goals (Next 1-2 weeks)
- [ ] Goal 1: [Description]
- [ ] Goal 2: [Description]
- [ ] Goal 3: [Description]

### Architecture Decisions Made
Document any significant architectural decisions:
- **Decision**: [What was decided]
- **Rationale**: [Why this approach]
- **Alternatives**: [What else was considered]
- **Impact**: [How it affects future development]

## 📚 Key Resources for Next Team

### Essential Reading (in order)
1. `PROJECT_TRACKING.md` - Current project status
2. `.kiro/specs/adx-core/development-kickoff/immediate-start-guide.md` - Foundation overview
3. `.kiro/specs/adx-core/development-kickoff/next-development-phases.md` - Future roadmap
4. [Any other relevant specs]

### Code Areas to Focus On
- **Primary**: [Main area of development]
- **Secondary**: [Supporting areas]
- **Dependencies**: [External dependencies to be aware of]

### Testing Strategy
- **Unit Tests**: [Location and how to run]
- **Integration Tests**: [Location and how to run]
- **Manual Testing**: [Key scenarios to test]

## 🔧 Development Environment Notes

### Special Setup Requirements
- [ ] No special requirements
- [ ] Custom environment variables needed: [List them]
- [ ] Additional tools required: [List them]
- [ ] Special configuration: [Describe]

### Performance Notes
- **Build Time**: [Typical build duration]
- **Test Time**: [Typical test duration]
- **Known Slow Areas**: [Any performance bottlenecks]

## 📞 Handoff Meeting Notes

### Meeting Details
- **Date**: [Meeting date]
- **Duration**: [Meeting length]
- **Participants**: [Who attended]

### Key Discussion Points
- Point 1: [Summary]
- Point 2: [Summary]
- Point 3: [Summary]

### Action Items
- [ ] **Action**: [Description] - **Owner**: [Who] - **Due**: [When]
- [ ] **Action**: [Description] - **Owner**: [Who] - **Due**: [When]

## ✅ Handoff Verification

### Incoming Team Checklist
- [ ] I have read all the essential documentation
- [ ] I have successfully started the development environment
- [ ] I have verified all services are working
- [ ] I have reviewed the current codebase
- [ ] I understand the immediate priorities
- [ ] I have updated `PROJECT_TRACKING.md` with my team info

### Final Sign-off
- **Outgoing Team**: [Signature/confirmation]
- **Incoming Team**: [Signature/confirmation]
- **Date**: [Handoff completion date]

---

**Note**: After completing this handoff, update the `PROJECT_TRACKING.md` file with the new team information and current status.