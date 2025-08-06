# Architecture Overview

This document provides a high-level overview of the Bevy GTA Clone's architecture and how the different modules work together.

## Core Architecture Principles

### Simplicity First
- Clear separation of concerns between modules
- Minimal coupling between components
- Straightforward data flow that's easy to trace
- No complex inheritance hierarchies

### Bevy ECS Patterns
- **Components**: Pure data structures (no behavior)
- **Systems**: Pure functions that operate on components
- **Resources**: Shared state across systems
- **Events**: Communication between plugins
- **Plugins**: Self-contained modules with clear interfaces

## System Execution Flow

```mermaid
graph TD
    A[ServiceInit] --> B[WorldSetup]
    B --> C[SecondarySetup] 
    C --> D[ServiceUpdates]
    
    A --> A1[Distance Cache]
    A --> A2[Entity Limits]
    A --> A3[Timing Services]
    
    B --> B1[Terrain Generation]
    B --> B2[Road Networks]
    B --> B3[Physics World]
    
    C --> C1[NPCs & Vehicles]
    C --> C2[Vegetation]
    C --> C3[Interactive Objects]
    
    D --> D1[Entity Culling]
    D --> D2[Physics Updates]
    D --> D3[Input Processing]
    D --> D4[UI Updates]
```

## Plugin Communication Architecture

```mermaid
graph LR
    subgraph "Input Layer"
        IP[Input Plugin]
    end
    
    subgraph "Core Layer"
        GC[Game Core]
        GS[Game Setup]
    end
    
    subgraph "Gameplay Layer"
        PP[Player Plugin]
        VP[Vehicle Plugin]
        WP[World Plugin]
        VLP[Vegetation Plugin]
    end
    
    subgraph "Interface Layer"
        UI[UI Plugin]
        WA[Water Plugin]
    end
    
    subgraph "Utility Layer"
        PE[Persistence Plugin]
    end
    
    IP --> PP
    PP --> VP
    PP --> WP
    VP --> VLP
    WP --> VLP
    UI --> GC
    PE --> GC
    
    PP -.->|Events| VP
    VP -.->|Events| WP
    WP -.->|Events| UI
```

## Module Responsibilities

### Components (`src/components/`)
- **Purpose**: Pure data structures that store entity state
- **Rules**: No behavior, only data fields
- **Examples**: `PlayerComponent`, `VehicleComponent`, `LodComponent`

### Systems (`src/systems/`)
- **Purpose**: Pure functions that operate on components
- **Rules**: Single responsibility, communicate via events/resources
- **Categories**: Gameplay, World Management, Services, Interface

### Plugins (`src/plugins/`)
- **Purpose**: Self-contained modules that manage specific game features
- **Rules**: Event-based communication only, no direct dependencies
- **Types**: Core, Gameplay, Interface, Utility

### Factories (`src/factories/`)
- **Purpose**: Consistent entity creation patterns
- **Rules**: Stateless functions, input validation, default values
- **Benefits**: Consistency, maintainability, testability

## Data Flow Patterns

### Entity Creation Flow
```mermaid
sequenceDiagram
    participant S as System
    participant F as Factory
    participant C as Commands
    participant W as World
    
    S->>F: Request entity creation
    F->>F: Validate inputs
    F->>F: Apply defaults
    F->>C: Spawn entity with components
    C->>W: Entity added to world
    W->>S: Entity available for queries
```

### Event Communication Flow
```mermaid
sequenceDiagram
    participant P1 as Plugin A
    participant E as Event System
    participant P2 as Plugin B
    
    P1->>E: Send event
    E->>E: Queue event
    E->>P2: Deliver event (next frame)
    P2->>P2: Process event
    P2->>E: Optional response event
```

## Performance Optimization Strategy

### Distance-Based Culling
- **Buildings**: 300m range
- **Vehicles**: 150m range  
- **NPCs**: 100m range
- **Cache**: 5-frame cache with 2048 entry limit

### System Timing Intervals
- **Road Generation**: 0.5s intervals
- **Dynamic Content**: 2.0s intervals
- **Entity Culling**: 0.5s intervals

### Spawn Rate Limits (Ultra-Reduced)
- **Buildings**: 8% spawn rate
- **Vehicles**: 4% spawn rate
- **Trees**: 5% spawn rate
- **NPCs**: 1% spawn rate

## Development Guidelines

### Adding New Features
1. **Identify the domain**: Which plugin should own this feature?
2. **Design components**: What data structures are needed?
3. **Create systems**: What functions operate on this data?
4. **Define events**: How does this communicate with other plugins?
5. **Use system sets**: Which execution phase does this belong to?

### Maintaining Simplicity
- Avoid tangled interdependencies between modules
- Keep functions focused on single responsibilities
- Use clear, direct APIs between components
- Prefer composition over complex inheritance
- Make data flow easy to trace

### Testing Strategy
- **Unit Tests**: Test individual systems and components
- **Integration Tests**: Test plugin interactions
- **Performance Tests**: Verify frame rate targets
- **Pattern**: Use `App::new().add_plugins(MinimalPlugins)` for Bevy tests

## Common Patterns

### System Registration
```rust
app.add_systems(Update, (
    my_system.in_set(GameSystemSets::ServiceUpdates),
    another_system.in_set(GameSystemSets::WorldSetup),
));
```

### Event Communication
```rust
// Sending events
fn sender_system(mut events: EventWriter<MyEvent>) {
    events.send(MyEvent { data: "hello" });
}

// Receiving events  
fn receiver_system(mut events: EventReader<MyEvent>) {
    for event in events.read() {
        // Process event
    }
}
```

### Resource Access
```rust
fn system_with_resources(
    mut shared_resource: ResMut<MyResource>,
    read_only_resource: Res<AnotherResource>,
) {
    // Access shared state
}
```

This architecture enables scalable development while maintaining the core principle of simplicity throughout the codebase.
