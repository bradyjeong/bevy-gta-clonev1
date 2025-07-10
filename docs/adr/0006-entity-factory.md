# ADR-0006: Entity Factory Pattern

## Status
Accepted

## Context

The game requires a flexible system for creating entities from predefined templates (prefabs). This system must support:

- **Rapid Prototyping**: Designers need to create and modify entity templates without code changes
- **Performance**: Entity spawning must be fast enough for real-time gameplay
- **Maintainability**: Component definitions should be easy to understand and modify
- **Hot-Reload**: Development workflow requires live reloading of prefab definitions
- **Type Safety**: Runtime errors should be minimized through compile-time validation where possible

The main architectural decision is between:
1. **Compile-time Generation**: Code-generated factories from static definitions
2. **Runtime Interpretation**: Dynamic loading and interpretation of prefab data
3. **Hybrid Approach**: Static registration with dynamic data loading

## Decision

We will implement a **Factory Pattern** with **RON-based prefab definitions** and **runtime component initialization**.

### Architecture Components

```rust
// Core factory pattern
pub struct Factory {
    registry: HashMap<PrefabId, Prefab>,
}

// Prefab definition with component initializers
pub struct Prefab {
    components: Vec<Box<dyn ComponentInit>>,
}

// Component initialization trait
pub trait ComponentInit: Send + Sync {
    fn init(&self, cmd: &mut Commands, entity: Entity) -> Result<(), Error>;
    fn as_any(&self) -> &dyn Any;
}

// RON-based loading (feature-gated)
#[cfg(feature = "ron")]
pub struct RonLoader {
    content: String,
}
```

### Data Format Choice: RON (Rusty Object Notation)

RON was chosen over alternatives for these reasons:

**Advantages:**
- **Rust Native**: Seamless integration with Rust type system
- **Human Readable**: Easy for designers to read and modify
- **Comments**: Supports documentation within data files
- **Type Safety**: Strong typing with compile-time validation
- **Flexibility**: Supports complex nested structures

**Example RON Prefab:**
```ron
RonPrefab(
    components: [
        RonComponent(
            component_type: "Transform",
            data: Map({
                "translation": Map({"x": 0.0, "y": 0.0, "z": 0.0}),
                "rotation": Map({"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0}),
                "scale": Map({"x": 1.0, "y": 1.0, "z": 1.0})
            })
        ),
        RonComponent(
            component_type: "Health",
            data: Number(100.0)
        )
    ]
)
```

## Alternatives Considered

### Alternative 1: Compile-time Code Generation

**Approach**: Generate Rust code from YAML/JSON definitions at build time

**Pros:**
- **Maximum Performance**: Zero runtime overhead
- **Compile-time Validation**: All errors caught at build time
- **Type Safety**: Full Rust type system available
- **IDE Support**: Code completion and refactoring

**Cons:**
- **Build Complexity**: Complex build scripts and codegen
- **Inflexibility**: Requires rebuilds for data changes
- **Designer Friction**: Technical barrier for non-programmers
- **Hot-reload Difficulty**: Complex to implement live updates

**Verdict**: Rejected - Too rigid for rapid iteration needs

### Alternative 2: Scriptable Entity System

**Approach**: Lua/JavaScript scripting for entity definitions

**Pros:**
- **Maximum Flexibility**: Full programming language available
- **Designer Friendly**: Familiar scripting languages
- **Hot-reload**: Natural scripting environment support
- **Runtime Modification**: Entities can be modified at runtime

**Cons:**
- **Performance Overhead**: Scripting runtime costs
- **Error Handling**: Runtime errors difficult to debug
- **Security Concerns**: Arbitrary code execution risks
- **Integration Complexity**: FFI and binding maintenance

**Verdict**: Rejected - Performance and security concerns

### Alternative 3: Binary Prefab Format

**Approach**: Custom binary format with editor tools

**Pros:**
- **Maximum Performance**: Minimal parsing overhead
- **Compact Size**: Smaller file sizes
- **Version Control**: Structured binary diffs possible
- **Tool Integration**: Custom editors possible

**Cons:**
- **Tooling Burden**: Custom editors required
- **Debugging Difficulty**: Binary files not human-readable
- **Merge Conflicts**: Binary files difficult to merge
- **Format Evolution**: Binary format changes break compatibility

**Verdict**: Rejected - Too much tooling overhead

## Implementation Strategy

### Phase 1: Core Factory (Completed)
- ✅ `Factory` struct with HashMap registry
- ✅ `Prefab` struct with component initializers
- ✅ `ComponentInit` trait for initialization
- ✅ Basic error handling and validation

### Phase 2: RON Integration (Completed)
- ✅ `RonLoader` for file-based prefab loading
- ✅ `RonPrefab` and `RonComponent` structures
- ✅ Feature-gated RON support
- ✅ Comprehensive test coverage

### Phase 3: Component Registry (Future)
- 🔄 Component type registration system
- 🔄 Automatic component deserialization
- 🔄 Validation and error reporting
- 🔄 Hot-reload infrastructure

### Phase 4: Tool Integration (Future)
- 🔄 Asset pipeline integration
- 🔄 Editor support for prefab editing
- 🔄 Live preview and debugging
- 🔄 Performance profiling tools

## Trade-offs Analysis

### Performance Trade-offs

**Accepted:**
- **Runtime Parsing**: RON parsing adds ~10-50μs per prefab load
- **Dynamic Dispatch**: Virtual calls through `ComponentInit` trait
- **Memory Overhead**: HashMap storage and boxed components

**Mitigated:**
- **Caching**: Prefabs are loaded once and reused
- **Batching**: Multiple entities can be spawned from same prefab
- **Lazy Loading**: Prefabs loaded on-demand

### Flexibility Trade-offs

**Accepted:**
- **Type Erasure**: Components stored as trait objects
- **Validation Timing**: Some errors only caught at runtime
- **Schema Evolution**: RON format changes require migration

**Mitigated:**
- **Error Handling**: Comprehensive error types and recovery
- **Validation**: Early validation during prefab loading
- **Backward Compatibility**: Careful RON schema evolution

### Development Trade-offs

**Accepted:**
- **Learning Curve**: Developers must learn RON syntax
- **Debugging**: Less IDE support for RON files
- **Tool Dependency**: Requires RON parser maintenance

**Mitigated:**
- **Documentation**: Comprehensive RON format documentation
- **Examples**: Rich example prefabs and tutorials
- **Validation**: Clear error messages for malformed RON

## Success Metrics

### Performance Targets
- **Prefab Loading**: <100μs per prefab from RON
- **Entity Spawning**: <10μs per entity from registered prefab
- **Memory Usage**: <1KB overhead per registered prefab
- **Hot-reload**: <500ms for prefab file changes

### Quality Targets
- **Test Coverage**: >80% line coverage
- **Error Handling**: All error paths tested
- **Documentation**: All public APIs documented
- **Examples**: Working examples for all features

### Usability Targets
- **Designer Workflow**: RON editing without programmer assistance
- **Error Messages**: Clear, actionable error reporting
- **Hot-reload**: Seamless development experience
- **Performance**: 60+ FPS with 1000+ entities

## Future Considerations

### Potential Improvements
1. **Visual Editor**: GUI tool for prefab editing
2. **Component Validation**: Compile-time validation for common components
3. **Binary Cache**: Cached binary format for faster loading
4. **Streaming**: Async prefab loading for large worlds

### Migration Path
- **V1 → V2**: Add binary cache layer while maintaining RON support
- **V2 → V3**: Add visual editor with RON export/import
- **V3 → V4**: Consider compile-time validation for performance-critical paths

### Risk Mitigation
- **RON Dependency**: Pin RON version, maintain fork if needed
- **Performance Regression**: Continuous benchmarking and profiling
- **Format Evolution**: Versioned RON schemas with migration tools
- **Tool Maintenance**: Dedicated tooling team for editor support

## Conclusion

The Factory Pattern with RON-based prefab definitions provides the optimal balance of:
- **Performance**: Fast enough for real-time gameplay
- **Flexibility**: Rapid iteration and hot-reload support
- **Maintainability**: Clear separation of data and code
- **Type Safety**: Rust's type system with runtime validation

This approach supports the game's requirements for rapid prototyping while maintaining production-quality performance and reliability.
