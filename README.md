# Amp Game Engine

![CI Status](https://github.com/bradyjeong/bevy-gta-clone/workflows/CI/badge.svg)
![Test Coverage](https://codecov.io/gh/bradyjeong/bevy-gta-clone/branch/main/graph/badge.svg)
![Rust Version](https://img.shields.io/badge/rust-1.77+-blue.svg)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)

A AAA-level open world game engine built with Rust and Bevy, optimized for Amp development workflows.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/bradyjeong/bevy-gta-clone.git
cd bevy-gta-clone

# Build the workspace
cargo build --workspace

# Run the minimal example
cargo run --bin minimal

# Run tests
cargo test --workspace

# Run full CI pipeline locally
cargo xtask ci
```

## Features

- 🌍 **Hierarchical Spatial Partitioning** - Efficient world streaming with Morton encoding
- 🎮 **GPU-Driven Rendering** - Modern wgpu-based rendering pipeline
- ⚡ **High Performance** - 60+ FPS target with object pooling and minimal allocations
- 🧪 **Comprehensive Testing** - 78+ unit tests with property-based testing
- 🔧 **Developer Experience** - Fast compilation, hot reloading, integrated tooling
- 📊 **Performance Monitoring** - Built-in profiling and frame analysis

## Architecture

The engine is built with a clean multi-crate architecture:

```
├─ crates/
│   ├─ amp_core/          # Core error handling and utilities
│   ├─ amp_math/          # Spatial mathematics and Morton encoding  
│   ├─ amp_spatial/       # Hierarchical spatial partitioning
│   ├─ amp_gpu/           # GPU abstraction over wgpu
│   ├─ amp_world/         # ECS world management
│   └─ [future crates]    # amp_physics, amp_ai, amp_render
├─ examples/              # Example applications
├─ tools/xtask/           # Development automation
└─ docs/                  # Documentation
```

See [Architecture Overview](docs/architecture/README.md) for detailed technical information.

## Development

### Prerequisites

- Rust 1.77+ (install via [rustup](https://rustup.rs/))
- Git

### Building

```bash
# Check all crates compile
cargo check --workspace

# Build with optimizations
cargo build --release --workspace

# Run linting
cargo clippy --workspace --all-targets --all-features

# Format code
cargo fmt --all
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo llvm-cov --workspace --lcov --output-path lcov.info

# Run specific crate tests
cargo test -p amp_math
```

### Development Tools

```bash
# Run full CI pipeline locally
cargo xtask ci

# Format all code
cargo xtask fmt

# Run tests only
cargo xtask test

# Generate documentation
cargo xtask doc
```

## Documentation

- **[Development Guide](docs/guides/development.md)** - Setup and workflow
- **[Architecture Overview](docs/architecture/README.md)** - Technical design
- **[Contributing Guidelines](CONTRIBUTING.md)** - How to contribute
- **[API Documentation](https://docs.rs/amp-game-engine)** - Generated API docs

## Performance

The engine targets AAA-level performance:

- **60+ FPS** on desktop platforms
- **Distance-based culling** (buildings 300m, vehicles 150m, NPCs 100m)
- **Object pooling** and per-frame arenas
- **Parallel execution** with Bevy's ECS scheduler
- **GPU-driven rendering** with indirect draw calls

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
