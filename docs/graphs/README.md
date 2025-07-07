# Dependency Graphs

This directory contains visual representations of the project's architecture and dependencies.

## Files

- `crate_dependencies.dot` - GraphViz diagram showing crate-to-crate dependencies
- `system_flow.dot` - Bevy system execution flow and dependencies

## Viewing

To convert DOT files to images:

```bash
# SVG format
dot -Tsvg crate_dependencies.dot > crate_dependencies.svg

# PNG format  
dot -Tpng crate_dependencies.dot > crate_dependencies.png
```

## Architecture Overview

The project follows a layered architecture:

1. **Engine Layer** (`engine_*`) - Core utilities and Bevy abstractions
2. **Gameplay Layer** (`gameplay_*`) - Domain-specific game logic
3. **Game Layer** (`game_*`) - Integration and application entry point

Dependencies flow upward through the layers, with no circular dependencies between layers.
