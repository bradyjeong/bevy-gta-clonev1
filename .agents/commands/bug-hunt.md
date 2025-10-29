Spawn multiple subagents to discover bugs across the entire codebase, then analyze and report findings.

Strategy:
- Launch parallel subagents to analyze different modules (components/, systems/, plugins/, factories/)
- Each subagent searches for: logic errors, race conditions, panics/unwraps, physics edge cases, asset loading issues, memory leaks, collision detection bugs, state inconsistencies, fighting systems
- Aggregate findings and categorize by severity (critical/high/medium/low)
- Provide detailed report with file locations, bug descriptions, and suggested fixes
- Consult the oracle for complex analysis if needed

Focus areas:
- Physics validation (velocity clamping, collision handling, ground detection)
- Asset loading and error handling (RON parsing, mesh/collider consistency)
- ECS component lifecycle (spawning, despawning, resource cleanup)
- Input handling edge cases (control state validation, vehicle transitions)
- Rendering and performance (culling bugs, particle system leaks)
