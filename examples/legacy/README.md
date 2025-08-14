# Legacy Input System

This directory contains the legacy input management system that has been replaced by asset-based controls.

## Moved Files
- `input_config.rs` - Legacy state-based input configuration  
- `input_manager.rs` - Legacy input processing with action mapping
- `vehicle_control_config.rs` - Legacy hardcoded vehicle control definitions

## Migration Status
**REPLACED BY**: Asset-based control system using RON configuration files
**CURRENT SYSTEM**: `src/systems/input/asset_based_controls.rs` + `assets/config/vehicle_controls.ron`

## Why Migrated
- Single source of truth (RON file vs scattered hardcoded maps)
- No code changes needed for new controls
- Runtime customization support
- Simplified data flow: RON → ControlState → Movement Systems

## If You Need Legacy Code
These files are preserved for reference but should not be used in new development.
Use the asset-based control system in `src/systems/input/asset_based_controls.rs` instead.
