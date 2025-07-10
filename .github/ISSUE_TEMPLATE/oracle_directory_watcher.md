# Directory Watcher System for Hot-Reload

## 📋 Task Overview
**Oracle Timeline:** Week 4 (sub-component of hot-reload)
**Dependencies:** Basic hot-reload infrastructure
**Estimated Effort:** 2-3 days

Implement a robust directory watcher system that monitors multiple directories for file changes and provides efficient event handling for the hot-reload system.

## 🎯 Goals
- [ ] Cross-platform directory monitoring
- [ ] Efficient recursive directory watching
- [ ] Event filtering and deduplication
- [ ] Integration with hot-reload manager
- [ ] Performance optimization for large directory trees

## 🔧 Technical Requirements

### Core Components
1. **Directory Watcher**
   ```rust
   pub struct DirectoryWatcher {
       watchers: HashMap<PathBuf, RecommendedWatcher>,
       event_tx: mpsc::Sender<WatchEvent>,
       debouncer: EventDebouncer,
   }
   ```

2. **Event Processing**
   ```rust
   pub enum WatchEvent {
       FileCreated(PathBuf),
       FileModified(PathBuf),
       FileDeleted(PathBuf),
       DirectoryCreated(PathBuf),
       DirectoryDeleted(PathBuf),
   }
   ```

3. **Event Debouncer**
   ```rust
   pub struct EventDebouncer {
       pending_events: HashMap<PathBuf, Instant>,
       debounce_duration: Duration,
   }
   ```

### Implementation Details
- Use `notify` crate for cross-platform file watching
- Implement recursive directory monitoring
- Add event filtering for relevant file types
- Support multiple directory roots simultaneously
- Efficient event deduplication and batching

## ✅ Acceptance Criteria

### Functional Requirements
- [ ] Monitor multiple directories recursively
- [ ] Detect file create/modify/delete events
- [ ] Filter events by file extension (.ron, .toml, etc.)
- [ ] Debounce rapid file changes (300ms default)
- [ ] Handle directory creation/deletion gracefully

### Quality Requirements
- [ ] Test coverage ≥70% for all watcher components
- [ ] Cross-platform compatibility (macOS, Linux, Windows)
- [ ] No resource leaks or zombie watchers
- [ ] Graceful error handling for missing directories
- [ ] Clean shutdown and resource cleanup

### Performance Requirements
- [ ] Event detection latency <100ms
- [ ] Support monitoring 10,000+ files efficiently
- [ ] Memory usage <5MB for watcher system
- [ ] No impact on file system performance
- [ ] Batch processing for multiple simultaneous events

## 📝 Implementation Plan

### Phase 1: Basic Watcher (1 day)
- Implement core DirectoryWatcher struct
- Add basic file event detection
- Create cross-platform abstraction layer
- Test with single directory monitoring

### Phase 2: Event Processing (1 day)
- Implement event debouncing system
- Add file type filtering
- Create event batching for efficiency
- Test with rapid file changes

### Phase 3: Multi-Directory Support (0.5 days)
- Add support for multiple watch roots
- Implement efficient event routing
- Test with complex directory structures
- Add directory creation/deletion handling

### Phase 4: Integration & Testing (0.5 days)
- Integrate with hot-reload manager
- Performance testing and optimization
- Error handling and edge cases
- Documentation and examples

## 🔗 Dependencies & Blockers

### Required Before Starting
- ✅ Basic hot-reload infrastructure planning
- ✅ File path abstractions

### Potential Blockers
- Platform-specific file watching limitations
- Performance with very large directory trees
- Complex symbolic link handling

## 📊 Success Metrics

### Reliability Metrics
- Event detection success rate >99%
- Zero missed file changes during normal operation
- Graceful handling of file system errors
- No resource leaks after 24-hour operation

### Performance Metrics
- Event latency <100ms for file changes
- Memory usage <5MB for 10,000 watched files
- CPU usage <1% during normal operation
- No measurable impact on file system performance

## 🛠️ Technical Considerations

### Cross-Platform Support
- Use `notify` crate's recommended watcher
- Handle platform-specific event types
- Abstract over different file system APIs
- Test on macOS, Linux, and Windows

### Event Filtering
- Filter by file extensions (.ron, .toml, .json)
- Ignore temporary files and editor backups
- Support custom filter patterns
- Efficient pattern matching for large file sets

### Error Handling
- Graceful handling of missing directories
- Recovery from file system errors
- Logging for debugging and monitoring
- Fallback strategies for watcher failures

### Performance Optimization
- Batch event processing to reduce overhead
- Efficient data structures for event tracking
- Lazy initialization of watchers
- Memory-efficient event storage

## 🔄 Related Issues
- Part of: Hot-reload implementation (Week 4)
- Enables: Config hot-reload, Prefab hot-reload
- Relates to: Asset pipeline integration

## 📚 References
- [notify crate documentation](https://docs.rs/notify/latest/notify/)
- [File watching best practices](https://github.com/notify-rs/notify/blob/main/examples/async_monitor.rs)
- [Cross-platform file system events](https://docs.rs/notify/latest/notify/event/index.html)

## 🎮 Usage Example
```rust
// Create directory watcher
let watcher = DirectoryWatcher::new()
    .watch_directory("assets/config/", &["toml", "ron"])
    .watch_directory("assets/prefabs/", &["ron"])
    .with_debounce(Duration::from_millis(300));

// Handle events
while let Ok(event) = watcher.recv() {
    match event {
        WatchEvent::FileModified(path) => {
            println!("File modified: {:?}", path);
        }
        WatchEvent::FileCreated(path) => {
            println!("File created: {:?}", path);
        }
        _ => {}
    }
}
```

## 🧪 Testing Strategy
- Unit tests for event detection and filtering
- Integration tests with real file system operations
- Performance tests with large directory trees
- Cross-platform compatibility testing
- Stress tests with rapid file changes

## 🔧 Configuration Options
```rust
pub struct WatcherConfig {
    pub debounce_duration: Duration,          // Default: 300ms
    pub recursive: bool,                      // Default: true
    pub follow_symlinks: bool,                // Default: false
    pub file_extensions: Vec<String>,         // Default: ["ron", "toml"]
    pub ignore_patterns: Vec<String>,         // Default: [".git", ".tmp"]
}
```
