# Week 3: Component Registry & Real RonComponent Deserialization

## 📋 Task Overview
**Oracle Timeline:** Week 3 (Oracle-identified priority)
**Dependencies:** gameplay_factory crate (Week 2 complete)
**Estimated Effort:** 5-7 days

Implement a comprehensive component registry system that enables automatic deserialization of RON components into actual Bevy components, replacing the current stub implementation.

## 🎯 Goals
- [ ] Complete component type registration system
- [ ] Implement automatic RON → Bevy component deserialization  
- [ ] Add validation and error reporting for component data
- [ ] Maintain 70%+ test coverage
- [ ] Ensure CI pipeline stays <20s build time

## 🔧 Technical Requirements

### Core Components
1. **Component Registry**
   ```rust
   pub struct ComponentRegistry {
       deserializers: HashMap<String, Box<dyn ComponentDeserializer>>,
       validators: HashMap<String, Box<dyn ComponentValidator>>,
   }
   ```

2. **Component Deserialization**
   ```rust
   pub trait ComponentDeserializer: Send + Sync {
       fn deserialize(&self, data: &ron::Value) -> Result<Box<dyn Component>, Error>;
       fn component_name(&self) -> &str;
   }
   ```

3. **Auto-Registration Macros**
   ```rust
   #[register_component]
   pub struct Transform {
       pub translation: Vec3,
       pub rotation: Quat,
       pub scale: Vec3,
   }
   ```

### Implementation Details
- Extend `gameplay_factory` crate with registry module
- Use procedural macros for component registration
- Implement deserializers for common Bevy components (Transform, Health, etc.)
- Add validation for component data integrity
- Support nested component structures

## ✅ Acceptance Criteria

### Functional Requirements
- [ ] Component registry can register/unregister component types
- [ ] RON data automatically deserializes to proper Bevy components
- [ ] Clear error messages for malformed component data
- [ ] Support for at least 10 common game components
- [ ] Validation catches type mismatches and invalid values

### Quality Requirements
- [ ] Test coverage ≥70% (current: 80.43%)
- [ ] All tests pass with `cargo test --workspace`
- [ ] No compilation warnings with `-Dwarnings`
- [ ] CI build time remains <20 seconds
- [ ] Documentation for all public APIs

### Performance Requirements
- [ ] Component deserialization <50μs per component
- [ ] Registry lookup <1μs per component type
- [ ] Memory usage <2KB per registered component type
- [ ] No runtime allocations during deserialization hot path

## 📝 Implementation Plan

### Phase 1: Registry Foundation (2 days)
- Create `ComponentRegistry` struct and core traits
- Implement basic registration/lookup functionality
- Add error handling for unknown component types
- Write comprehensive unit tests

### Phase 2: Deserialization System (2 days)
- Implement `ComponentDeserializer` trait
- Create deserializers for basic types (Transform, Health, etc.)
- Add RON value → component conversion logic
- Test with complex nested structures

### Phase 3: Validation & Error Handling (1 day)
- Implement component data validation
- Add detailed error messages with context
- Create validation framework for custom components
- Test error scenarios and recovery

### Phase 4: Integration & Testing (1 day)
- Integrate with existing Factory system
- Update minimal example to use real components
- Performance testing and optimization
- Documentation and examples

## 🔗 Dependencies & Blockers

### Required Before Starting
- ✅ gameplay_factory crate (Week 2 complete)
- ✅ RON loader implementation
- ✅ Factory pattern established

### Potential Blockers
- Complex component types may need custom deserializers
- Performance requirements may require optimization
- Bevy version compatibility issues

## 📊 Success Metrics

### Development Metrics
- Component registration time <100μs
- Deserialization success rate >99%
- Error message clarity score >8/10 (dev feedback)
- Test coverage maintained above 70%

### Integration Metrics
- minimal example runs with real components
- No performance regression in entity spawning
- CI pipeline remains green throughout development
- Documentation completeness >95%

## 🔄 Related Issues
- Depends on: gameplay_factory implementation (Week 2)
- Enables: Hot-reload implementation (Week 4)
- Relates to: Async prefab loading (Week 5+)

## 📚 References
- [ADR-0006: Entity Factory Pattern](../docs/adr/0006-entity-factory.md)
- [Oracle Consultations](../docs/oracle-consultations.md)
- [Bevy Component Documentation](https://docs.rs/bevy/latest/bevy/ecs/component/)
- [RON Specification](https://github.com/ron-rs/ron)
