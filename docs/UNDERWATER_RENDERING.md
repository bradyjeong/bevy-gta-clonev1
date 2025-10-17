# Underwater Rendering - Research & Implementation

## Scientific Background

### Light Absorption in Ocean Water

Based on oceanographic research, light absorption in water follows these patterns:

#### Wavelength Attenuation by Depth
- **Red (650nm)**: Completely absorbed in upper **10 meters**
- **Orange (600nm)**: Absorbed by **40 meters**
- **Yellow (580nm)**: Disappears before **100 meters**
- **Blue (440nm) & Green (550nm)**: Penetrate deepest (present at 100-200m)

#### General Light Attenuation (Beer-Lambert Law)
- At **1m depth**: 45% of surface light remains
- At **10m depth**: 16% of surface light remains
- At **100m depth**: 1% of surface light remains
- **Below 1000m**: No light (aphotic/midnight zone)

### Why Ocean Water Appears Blue

1. **Blue light** penetrates deepest and is scattered by water molecules
2. **Red, orange, yellow** are absorbed quickly in shallow water
3. **Coastal waters** appear green due to phytoplankton absorbing blue/red light

## Implementation Parameters

### Current Settings (Research-Based)

```rust
UnderwaterSettings {
    sea_level: 0.0,
    fog_density: 0.25,                      // Moderate density for clear ocean
    absorption: Vec3::new(0.8, 0.3, 0.15),  // R=0.8, G=0.3, B=0.15
    scatter_color: Vec3::new(0.02, 0.35, 0.48), // Deep blue-cyan
    enabled: 1,
}
```

### Parameter Explanations

#### `fog_density` (0.1 - 0.5 typical range)
- **0.1**: Very clear ocean (tropical, 20m+ visibility)
- **0.25**: Clear ocean (current setting)
- **0.35**: Moderate turbidity (coastal waters)
- **0.5+**: Murky/turbid water (harbors, river mouths)

#### `absorption` (RGB channels, 0.0 - 1.0)
Simulates wavelength-dependent light absorption:
- **R (0.8)**: High red absorption (realistic - red disappears at 10m)
- **G (0.3)**: Moderate green absorption
- **B (0.15)**: Low blue absorption (blue penetrates deepest)

**Rule**: RED >> GREEN > BLUE for realistic ocean

#### `scatter_color` (RGB, 0.0 - 1.0)
The color tint applied underwater:
- **R (0.02)**: Minimal red (red light already absorbed)
- **G (0.35)**: Moderate green-cyan
- **B (0.48)**: Strong blue (dominant color at depth)

**Result**: Deep blue-cyan color typical of clear ocean at 5-20m depth

## Shader Implementation

### Per-Pixel Depth-Based Fog

The shader calculates fog intensity based on:

```wgsl
let thickness = max(0.0, params.sea_level - world_pos.y);
let T = exp(-(params.absorption * thickness) * params.fog_density);
let fogged = color.rgb * T + params.scatter_color * (vec3<f32>(1.0) - T);
```

**How it works:**
1. **thickness**: Distance below sea level (0 at surface, increases with depth)
2. **T (transmission)**: Exponential falloff (Beer-Lambert law approximation)
3. **fogged**: Blend between original color and scatter color based on depth

### Smooth Transition

- **Above water (Y > 0)**: No effect (original rendering)
- **At surface (Y ≈ 0)**: Minimal tint, mostly visible
- **Shallow (0-10m)**: Gradual blue-green tint
- **Deep (10m+)**: Strong blue dominance, reduced visibility

## Tuning Guide

### For Different Water Types

#### **Tropical Clear Water** (Caribbean, Maldives)
```rust
fog_density: 0.15,
absorption: Vec3::new(0.7, 0.25, 0.1),
scatter_color: Vec3::new(0.05, 0.45, 0.6),  // Bright turquoise
```

#### **Open Ocean** (Atlantic, Pacific)
```rust
fog_density: 0.25,
absorption: Vec3::new(0.8, 0.3, 0.15),
scatter_color: Vec3::new(0.02, 0.35, 0.48),  // Deep blue (current)
```

#### **Coastal/Green Water**
```rust
fog_density: 0.35,
absorption: Vec3::new(0.6, 0.25, 0.2),
scatter_color: Vec3::new(0.08, 0.4, 0.35),  // Green-cyan
```

#### **Murky Harbor Water**
```rust
fog_density: 0.5,
absorption: Vec3::new(0.5, 0.35, 0.3),
scatter_color: Vec3::new(0.15, 0.3, 0.25),  // Dark green-brown
```

## References

1. **NOAA Ocean Explorer**: Light and Color in the Deep Sea
2. **Oceanography Textbooks**: Light attenuation in seawater
3. **Game Development**: Subnautica, Sea of Thieves underwater rendering
4. **Scientific Papers**: Beer-Lambert law application to ocean optics

## Future Enhancements

- [ ] Depth-based parameter variation (darker at greater depths)
- [ ] Caustics (light patterns from surface waves)
- [ ] Particle effects (suspended sediment, plankton)
- [ ] God rays (volumetric light shafts)
- [ ] Temperature-based color shifts (warm vs cold water)
