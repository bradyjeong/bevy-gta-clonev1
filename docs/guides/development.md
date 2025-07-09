# Development Guide

This guide covers the development workflow for the Amp Game Engine.

## Getting Started

### Prerequisites

- Rust 1.77+ (install via [rustup](https://rustup.rs/))
- Git
- VS Code with rust-analyzer (recommended)

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/bradyjeong/bevy-gta-clone.git
cd bevy-gta-clone

# Build all crates
cargo build --workspace

# Run tests to verify setup
cargo test --workspace

# Run the minimal example
cargo run --bin minimal
```

## Development Workflow

### Daily Development

1. **Start with checks**: `cargo xtask ci` runs the full pipeline locally
2. **Make changes**: Edit code in your preferred editor
3. **Test frequently**: `cargo test -p <crate-name>` for specific crates
4. **Check compilation**: `cargo check --workspace` for fast feedback
5. **Format code**: `cargo fmt --all` before committing

### Branch Strategy

- `main` - Stable, production-ready code
- `develop` - Integration branch for features
- `feature/*` - Individual feature branches
- `hotfix/*` - Critical bug fixes

### Commit Guidelines

Follow conventional commits format:

```
type(scope): brief description

Longer description if needed

- List specific changes
- Reference issue numbers
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `perf`

### Pull Request Process

1. Create feature branch: `git checkout -b feature/my-feature`
2. Make changes and add tests
3. Run full CI: `cargo xtask ci`
4. Push and create PR
5. Address review feedback
6. Squash and merge

## Architecture Guidelines

### Crate Organization

Each crate has a specific purpose:

- **amp_core**: Foundation types, error handling
- **amp_math**: Mathematical operations, spatial indexing
- **amp_spatial**: Hierarchical world management
- **amp_gpu**: Graphics abstraction
- **amp_world**: ECS integration

### Dependency Rules

- No circular dependencies
- Lower-level crates don't depend on higher-level ones
- External dependencies managed in workspace Cargo.toml

### Code Style

- Use `snake_case` for variables and functions
- Use `PascalCase` for types and structs
- Prefer `Result<T, E>` over `Option<T>` for error cases
- Document all public APIs
- Write tests for all public functions

## Testing Strategy

### Unit Tests

- Place tests in the same file as the code
- Use `#[cfg(test)]` modules
- Test both success and failure cases
- Aim for 70%+ code coverage

### Integration Tests

- Place in `tests/` directory at crate root
- Test public APIs and workflows
- Include performance benchmarks

### Property Testing

- Use `proptest` for mathematical functions
- Test invariants and edge cases
- Focus on spatial and numerical code

## Performance Guidelines

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Profile specific benchmark
cargo bench --bench spatial_benchmarks
```

### Optimization Workflow

1. **Measure first**: Use `cargo bench` to establish baseline
2. **Profile**: Use `perf` or `cargo flamegraph` to find hotspots
3. **Optimize**: Make targeted improvements
4. **Verify**: Ensure benchmarks improve
5. **Test**: Verify correctness is maintained

### Common Patterns

- Use object pools for frequently allocated objects
- Prefer stack allocation over heap when possible
- Use SIMD operations for mathematical code
- Cache expensive calculations

## Debugging

### Debug Builds

```bash
# Build with debug info
cargo build

# Run with debug logs
RUST_LOG=debug cargo run --bin minimal
```

### Release Builds

```bash
# Build optimized
cargo build --release

# Profile release build
cargo flamegraph --bin minimal
```

### Common Issues

- **Slow compilation**: Use `cargo check` for faster feedback
- **Test failures**: Run `cargo test -- --nocapture` for full output
- **Performance issues**: Profile with `cargo bench` first

## Documentation

### API Documentation

```bash
# Generate docs
cargo doc --workspace --no-deps

# Open in browser
cargo doc --workspace --no-deps --open
```

### Adding Documentation

- Document all public items
- Include examples in doc comments
- Use `cargo test --doc` to test examples

## Release Process

### Version Management

- Use semantic versioning (MAJOR.MINOR.PATCH)
- Update versions in workspace Cargo.toml
- Create git tags for releases

### Release Checklist

1. Update CHANGELOG.md
2. Run full test suite: `cargo test --workspace`
3. Update version numbers
4. Create release PR
5. Tag release after merge
6. Publish crates (if applicable)

## Getting Help

- **Documentation**: Check [docs/](../) directory
- **Issues**: Create GitHub issues for bugs/features
- **Discussions**: Use GitHub discussions for questions
- **Team Chat**: Internal team communication channels

## Tools and Editor Setup

### VS Code

Recommended extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Better TOML
- GitLens

### Command Line Tools

```bash
# Install useful tools
cargo install cargo-watch cargo-expand cargo-udeps
cargo install cargo-flamegraph cargo-llvm-cov

# Watch for changes
cargo watch -x "check --workspace"

# Check for unused dependencies
cargo udeps --workspace
```

## Troubleshooting

### Common Errors

**Compilation Errors**: 
- Run `cargo clean` and rebuild
- Check Rust version: `rustc --version`

**Test Failures**:
- Run single test: `cargo test test_name`
- Run with output: `cargo test -- --nocapture`

**Performance Issues**:
- Profile first: `cargo bench`
- Check for debug assertions in release builds

### Getting Unstuck

1. Check error messages carefully
2. Search GitHub issues
3. Ask for help in team channels
4. Create minimal reproduction case
