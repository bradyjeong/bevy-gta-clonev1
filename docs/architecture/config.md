# Configuration System Architecture

## Overview

The configuration system provides a hierarchical, type-safe approach to loading and managing application settings for the AMP Game Engine. Built on RON (Rust Object Notation) for human-readable configuration files, the system implements a structured search hierarchy that balances flexibility with sensible defaults.

## Architecture

### Core Components

#### Config Trait
```rust
pub trait Config: DeserializeOwned + Send + Sync + 'static + Default {
    const FILE_NAME: &'static str;
    
    fn default_path() -> PathBuf {
        PathBuf::from(Self::FILE_NAME)
    }
    
    fn embedded_defaults() -> Self {
        Self::default()
    }
    
    fn merge(self, other: Self) -> Self {
        other
    }
}
```

The `Config` trait defines the interface for configuration types:
- **Type Safety**: All configurations must implement `DeserializeOwned` for RON parsing
- **Thread Safety**: `Send + Sync` bounds enable safe concurrent access
- **Metadata**: `FILE_NAME` constant specifies the configuration file name
- **Path Resolution**: `default_path()` provides customizable file path logic
- **Embedded Defaults**: `embedded_defaults()` provides compile-time fallback values
- **Hierarchical Merging**: `merge()` enables custom merge behavior for configuration values

#### ConfigLoader
```rust
pub struct ConfigLoader {
    search_paths: Vec<PathBuf>,
}
```

The `ConfigLoader` manages configuration file discovery and loading:
- **Search Paths**: Ordered list of directories to search for configuration files
- **Hierarchical Resolution**: Implements priority-based file discovery
- **Caching**: Designed for future performance optimizations
- **Error Handling**: Comprehensive error reporting with context

## Search Hierarchy

The configuration system searches for files in the following order:

### 1. Current Working Directory (cwd)
- **Path**: `./config_file.ron`
- **Priority**: Highest
- **Use Case**: Project-specific overrides, development settings
- **Example**: `./graphics.ron` overrides system-wide graphics settings

### 2. XDG Config Directory
- **Path**: `$XDG_CONFIG_HOME/amp/config_file.ron`
- **Fallback**: `~/.config/amp/config_file.ron` (Unix systems)
- **Priority**: Medium
- **Use Case**: User-specific settings, persistent preferences
- **Example**: `~/.config/amp/input.ron` for user input mappings

### 3. Embedded Defaults
- **Priority**: Lowest
- **Use Case**: Fallback values, initial setup
- **Status**: Planned for future implementation
- **Implementation**: Compile-time embedded RON strings

## API Usage

### Basic Configuration Loading

```rust
use config_core::{Config, ConfigLoader};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GraphicsConfig {
    resolution: (u32, u32),
    fullscreen: bool,
    vsync: bool,
    quality: String,
}

impl Config for GraphicsConfig {
    const FILE_NAME: &'static str = "graphics.ron";
}

// Load configuration
let loader = ConfigLoader::new();
let config: GraphicsConfig = loader.load_with_merge()?;
```

### Custom Configuration Paths

```rust
impl Config for AdvancedConfig {
    const FILE_NAME: &'static str = "advanced.ron";
    
    fn default_path() -> PathBuf {
        PathBuf::from("config/advanced.ron")
    }
}
```

### Configuration File Format (RON)

```ron
// graphics.ron
(
    resolution: (1920, 1080),
    fullscreen: false,
    vsync: true,
    quality: "high",
)
```

## Error Handling

The configuration system provides comprehensive error handling through the `amp_core::Error` type:

### Error Types

- **File Not Found**: Configuration file doesn't exist in any search path
- **Parse Error**: Invalid RON syntax or structure
- **I/O Error**: File system access issues
- **Validation Error**: Configuration values outside acceptable ranges

### Error Recovery

```rust
use amp_core::Result;

fn load_with_fallback() -> Result<GraphicsConfig> {
    let loader = ConfigLoader::new();
    
    match loader.load_with_merge::<GraphicsConfig>() {
        Ok(config) => Ok(config),
        Err(e) => {
            log::warn!("Failed to load graphics config: {}", e);
            // Return reasonable defaults
            Ok(GraphicsConfig {
                resolution: (1280, 720),
                fullscreen: false,
                vsync: true,
                quality: "medium".to_string(),
            })
        }
    }
}
```

## Best Practices

### Configuration Design

1. **Naming Convention**: Use descriptive, lowercase names with underscores
   - ✅ `graphics.ron`, `audio_settings.ron`
   - ❌ `cfg.ron`, `GameConfig.ron`

2. **Structure**: Group related settings logically
   ```ron
   (
       graphics: (
           resolution: (1920, 1080),
           quality: "high",
       ),
       audio: (
           master_volume: 0.8,
           sfx_volume: 0.9,
       ),
   )
   ```

3. **Validation**: Implement validation in the configuration struct
   ```rust
   #[derive(Deserialize, Debug)]
   struct AudioConfig {
       #[serde(deserialize_with = "validate_volume")]
       master_volume: f32,
   }
   
   fn validate_volume<'de, D>(deserializer: D) -> Result<f32, D::Error>
   where
       D: serde::Deserializer<'de>,
   {
       let volume = f32::deserialize(deserializer)?;
       if volume < 0.0 || volume > 1.0 {
           return Err(serde::de::Error::custom("Volume must be between 0.0 and 1.0"));
       }
       Ok(volume)
   }
   ```

### Performance Considerations

1. **Load Once**: Cache configuration objects after initial load
2. **Lazy Loading**: Only load configurations when needed
3. **Hot Reload**: Use file watching for development (planned feature)

### Development Workflow

1. **Local Development**: Place configuration files in the project root
2. **Testing**: Use temporary directories for isolated test configurations
3. **Distribution**: Package default configurations as embedded resources

## Integration Examples

### Game Engine Integration

```rust
use config_core::ConfigLoader;

struct GameEngine {
    graphics_config: GraphicsConfig,
    audio_config: AudioConfig,
    input_config: InputConfig,
}

impl GameEngine {
    fn new() -> Result<Self> {
        let loader = ConfigLoader::new();
        
        Ok(Self {
            graphics_config: loader.load_with_merge()?,
            audio_config: loader.load_with_merge()?,
            input_config: loader.load_with_merge()?,
        })
    }
}
```

### Bevy System Integration

```rust
use bevy::prelude::*;
use config_core::ConfigLoader;

fn setup_graphics_system(mut commands: Commands) {
    let loader = ConfigLoader::new();
    
    match loader.load_with_merge::<GraphicsConfig>() {
        Ok(config) => {
            commands.insert_resource(config);
        }
        Err(e) => {
            error!("Failed to load graphics config: {}", e);
            // Use defaults
        }
    }
}
```

## Future Enhancements

### Hot Reload (Planned)
- File system watching for configuration changes
- Automatic reload and notification system
- Development-only feature for fast iteration

### Embedded Defaults (Implemented)
- Compile-time embedded configuration files via `Config::embedded_defaults()`
- Fallback when no user configuration exists
- Ensures the application always has valid settings

### Configuration Validation (Planned)
- Runtime validation of configuration values
- Schema-based validation for complex configurations
- Better error messages with suggestions

### Environment Variable Support (Implemented)
- Override configuration values with `AMP_CONFIG` environment variable
- Support for containerized deployments
- Development and CI/CD integration

## Security Considerations

1. **File Permissions**: Configuration files should be readable only by the application user
2. **Sensitive Data**: Never store secrets or passwords in configuration files
3. **Path Traversal**: The system validates file paths to prevent directory traversal attacks
4. **Input Validation**: All configuration values should be validated before use

## Testing

The configuration system includes comprehensive tests covering:
- Configuration loading from various paths
- Error handling for missing and malformed files
- Search path priority and resolution
- RON deserialization edge cases

Run tests with:
```bash
cargo test --package config_core
```

## Related Systems

- **[amp_core](../api/amp_core/index.html)**: Error handling and shared utilities
- **[Logging](logging.md)**: Configuration for logging levels and outputs
- **[Asset Pipeline](assets.md)**: Asset loading configuration
- **[Rendering](rendering.md)**: Graphics settings and renderer configuration
