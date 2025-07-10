# Oracle Consultation Log

This document tracks key Oracle consultations and their strategic impact on the project.

## Consultation Format

Each consultation should include:
- **Date**: When the consultation occurred
- **Context**: What problem or decision prompted the consultation
- **Key Insights**: Most important strategic guidance
- **Actions Taken**: How the guidance was implemented
- **ADR Reference**: Link to any resulting Architecture Decision Records

## Consultations

### 2025-01-10: Strategic Shift to Bevy 0.16.1 Meta-Crate
**Context**: Current bevy_ecs 0.13 + micro-crate architecture creating ecosystem misalignment, test failures, development overhead

**Key Insights**:
- Current approach fights Bevy ecosystem, wastes time on solved problems (RON loaders, wgpu wrappers)
- Amp productivity comes from clear boundaries, not excessive crate count
- Cross-crate compilation overhead dominates CI time (40%+)
- Strategic 4-5 crate structure better than 6+ micro-crates
- Full Bevy 0.16.1 provides ecosystem leverage + future-proofing

**Actions Taken**:
- Created ADR-007 documenting strategic shift
- Updated Agent.md with Oracle's recommended structure
- Planned 10-14 day migration roadmap
- Target: amp_core + amp_math + amp_engine + amp_gameplay + amp_tools

**ADR Reference**: [ADR-0007](adr/0007-strategic-shift-bevy-meta-crate.md)

### 2025-01-07: Architecture Strategy Decision
**Context**: Choosing between clean restart, continued refactoring, or hybrid approach

**Key Insights**:
- Current codebase is 40% AAA implementation, 60% good architecture
- "Strangler-fig" hybrid approach optimal: extract proven systems, rebuild cleanly
- Multi-crate structure is correct direction but needs pruning
- Oracle estimates 2 months with disciplined execution

**Actions Taken**:
- Implemented 8-week extraction-based restart plan
- Created multi-crate workspace structure
- Established quality gates (no warnings, <60s compile, CI)

**ADR Reference**: [ADR-0002](adr/0002-oracle-guided-architecture.md)

### 2025-01-07: Week 1 Verification
**Context**: Verifying successful completion of foundation phase

**Key Insights**:
- Foundation is solid for Week 2 progression
- 78 tests passing with comprehensive coverage
- Minor polish items identified (coverage gate, publishing hygiene)
- Technical quality assessment: good algorithms, clean compilation

**Actions Taken**:
- Fixed documentation validation issues
- Implemented comprehensive documentation system
- Added automated validation to CI pipeline

**ADR Reference**: Documentation strategy captured in development workflows

---

## Usage Guidelines

### When to Consult Oracle
- Major architectural decisions
- Technology choice evaluation
- Performance optimization strategy
- Project milestone verification
- When stuck on complex technical problems

### When NOT to Consult Oracle
- Implementation details
- Minor bug fixes
- Routine development tasks
- Questions answered by existing documentation

### Documentation Process
1. **Consult Oracle** on strategic question
2. **Extract key insights** from response
3. **Document in this log** with context and actions
4. **Create ADR** for major architectural decisions
5. **Update AGENT.md** if workflow changes

## Benefits

- **Historical context** for future architectural decisions
- **Team alignment** on strategic direction
- **Decision rationale** preserved for new team members
- **Pattern recognition** for similar future problems
- **Oracle guidance** doesn't get lost in conversation history
