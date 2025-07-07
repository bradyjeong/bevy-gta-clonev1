# Golden Frame Testing

This directory contains reference images for golden frame testing - a form of visual regression testing that ensures rendering consistency across changes.

## How it works

1. **Reference frames**: PNG images that represent the expected rendering output for specific test scenarios
2. **Current frames**: Images captured during test runs
3. **Comparison**: Pixel-by-pixel comparison with configurable tolerance
4. **Diff images**: Generated when tests fail to show the differences

## Test scenarios

### basic_scene.png
- Static camera at position (10, 8, 10) looking at origin
- One red car (2×1×4 cuboid) at origin
- One blue building (3×5×3 cuboid) at (-5, 2.5, -3)
- Green ground plane (20×20) at y=-0.5
- Fixed directional light from (4, 8, 4)

### lod_transition.png
- Same scene but camera at distance (50, 10, 50)
- Tests LOD system behavior at far distances

### different_lighting.png
- Basic scene with warm lighting (reduced intensity, warmer color)
- Tests lighting variations

## Configuration

- **Epsilon tolerance**: 0.02 (2% pixel difference allowed)
- **Max different pixels**: 100 pixels
- **Resolution**: 800×600 pixels
- **Format**: PNG

## Running tests

```bash
cargo test golden_frame
```

## Updating reference frames

If rendering changes are intentional, delete the reference frames and re-run tests to generate new ones.

## Troubleshooting

- If tests fail, check the generated `*_diff.png` files to see what changed
- Diff images highlight differences in red
- Make sure graphics drivers are consistent between test environments
