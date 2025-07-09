# ADR-0002: Oracle-Guided Architecture Strategy

## Status
Accepted

## Context

The Oracle provided strategic guidance for the 8-week extraction-based restart, recommending a specific approach over alternative architectures (like Opus's "linked battleships" threading approach).

## Decision

We will follow the Oracle's "controlled greenfield extraction" strategy:

1. **Extract proven systems** from commits f430bc6 and d34cd28
2. **Use multi-crate workspace** structure for professional development
3. **Implement strict quality gates** (no warnings, determinism, <60s compile, CI)
4. **Follow 8-week milestone plan** with weekly checkpoints

The Oracle specifically recommended against:
- Complete restart from scratch (would lose validated work)
- Opus's complex threading architecture (3-4x effort for 5-15% gain)
- Continued incremental refactoring (would drag legacy ballast)

## Oracle's Key Insights

### Architecture Assessment
- Current foundation is 40% AAA-quality, 60% good architecture ideas
- Multi-crate structure is salvageable if pruned properly
- Documentation-driven development has created reality vs docs mismatch

### Strategic Recommendations
1. **Strangler-fig pattern**: Keep engine_core, RON configs, Rapier wrappers
2. **Create clean game crate**: Build new systems with amp guidelines
3. **Implement governance**: Freeze main, define "Amp Ready" checklist
4. **Phased approach**: Extract → CI hardening → vertical slice → features

### Performance Targets
- 60+ FPS with distance culling
- <60s compile time
- 70%+ test coverage
- Deterministic physics

## Consequences

### Positive
- Clear technical roadmap backed by strategic analysis
- Avoids both "restart paralysis" and "refactor fatigue"
- Maintains team velocity while improving architecture quality
- Proven approach validated by Oracle's codebase analysis

### Negative
- Requires discipline to follow Oracle's plan strictly
- Some existing code will be abandoned (but documented for reference)
- Weekly checkpoints create pressure for deliverable demos

## Implementation Notes

- Oracle consultations will be documented in ADRs for major decisions
- Weekly progress will be verified with Oracle when needed
- Success metrics defined by Oracle will be tracked continuously
