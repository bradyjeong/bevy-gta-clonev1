# 🏁 Bugatti Chiron Telemetry Dashboard

## Overview

The Bugatti Chiron Telemetry Dashboard is a premium real-time display system that shows advanced hypercar metrics when driving the SuperCar vehicle. This dashboard provides a luxury experience similar to what you'd find in a real $3M Bugatti Chiron.

## Features

### 🎛️ Real-Time Telemetry
- **Engine RPM & Gear**: Live engine speed and current gear position
- **Turbo System**: Quad-turbo W16 pressure, lag, and cooldown status
- **Performance Metrics**: Speed (mph/km/h), 0-60 acceleration timing
- **G-Force Readings**: Lateral and longitudinal G-forces
- **Systems Status**: Engine temperature, oil pressure monitoring
- **Driving Modes**: Comfort/Sport/Track mode display
- **Launch Control**: Advanced launch system status
- **Active Aerodynamics**: Wing angle and splitter position

### 🚗 Usage

#### Activation
- **Press F4** while driving a SuperCar to toggle the telemetry dashboard
- Only available when actively driving a SuperCar vehicle
- Automatically hides when exiting the vehicle or switching to non-SuperCar

#### Controls
- **F3**: Performance Monitor (general system performance)
- **F4**: Bugatti Telemetry (SuperCar-specific advanced metrics)

### 📊 Dashboard Layout

```
╔════════════════════════════════════╗
║          🏁 BUGATTI CHIRON W16         ║
║              TELEMETRY SYSTEM          ║
╠════════════════════════════════════╣
║                                        ║
║ 🚗 ENGINE & DRIVETRAIN                ║
║   RPM: 2500  / 6800  (37%)             ║
║   Gear: 3 / 7                          ║
║   Power: 1500 HP | Torque: 1180 Nm     ║
║                                        ║
║ 💨 TURBO SYSTEM (W16 QUAD-TURBO)      ║
║   Status: 3/4 TURBO                    ║
║   Pressure: 85% | Lag: 0.3s           ║
║   Cooldown: 2.1s / 5.0s                ║
║                                        ║
║ 🏎️  PERFORMANCE METRICS               ║
║   Speed: 145 mph (233 km/h)           ║
║   ⏱️  0-60: 2.4s (BEST)               ║
║                                        ║
║ 📊 G-FORCE TELEMETRY                  ║
║   Lateral: →1.2G | Long: ↑0.8G        ║
║   Traction: 92% | Downforce: 850N     ║
║                                        ║
║ 🔧 SYSTEMS STATUS                     ║
║   Mode: 🟡 SPORT                      ║
║   Launch Control: ⚡ READY             ║
║   Engine Temp: 🟢 75%  Oil: 🟢 85%    ║
║                                        ║
║ 🎛️  AERODYNAMICS                      ║
║   Wing Angle: 45% | Splitter: 30%     ║
║   Active Aero: ON                      ║
║                                        ║
╚════════════════════════════════════╝
```

### 🎨 Visual Design

#### Styling Features
- **Bugatti Signature Blue**: Premium cyan color scheme (0, 0.9, 1.0)
- **Luxury Layout**: Clean, professional hypercar aesthetic
- **Real-time Updates**: 10Hz refresh rate for smooth telemetry
- **Premium Typography**: Clear, readable font at 18px
- **Semi-transparent Background**: Elegant black background with blue border

#### Positioning
- **Left Side Placement**: Positioned at (20px, 50px) to avoid conflicts with F3 monitor
- **Fixed Position**: Absolute positioning for consistent display
- **Responsive Display**: Only visible when conditions are met

### 🔧 Technical Details

#### System Integration
- **Component-Based**: Uses SuperCar component data for all metrics
- **State-Aware**: Only displays during GameState::Driving
- **Performance Optimized**: Minimal overhead with 0.1s update intervals
- **Auto-Hide**: Intelligently hides when not applicable

#### Telemetry Metrics
- **Engine Data**: RPM, gear, power/torque readings
- **Turbo System**: 4-stage turbo with pressure/lag simulation
- **Physics**: Real G-force calculations from vehicle physics
- **Performance**: Tracked 0-60 times and launch metrics
- **Environmental**: Temperature and pressure monitoring
- **Aerodynamics**: Active wing and splitter positions

### 🚀 Advanced Features

#### Performance Timing
- **0-60 Acceleration**: Automatic timing when launching from standstill
- **Launch Control**: Status display for optimal acceleration
- **G-Force Monitor**: Real-time lateral and longitudinal forces

#### Engine Management
- **Rev Limiter**: Visual indication of RPM limits
- **Temperature Monitor**: Engine heat management display
- **Oil Pressure**: Critical system monitoring

#### Driving Modes
- 🟢 **Comfort**: Optimized for daily driving
- 🟡 **Sport**: Enhanced performance settings
- 🔴 **Track**: Maximum performance configuration
- 🔧 **Custom**: User-defined settings

### 💡 Tips for Best Experience

1. **Find the Bugatti**: Look for the SuperCar spawned in the world
2. **Use Launch Control**: Enable for optimal 0-60 times
3. **Monitor G-Forces**: Track cornering and acceleration performance
4. **Watch Turbo System**: Observe the quad-turbo staging
5. **Try Different Modes**: Switch between Comfort/Sport/Track for varied experience

### 🛠️ Implementation Details

#### Files Created/Modified
- `src/systems/ui/bugatti_telemetry.rs` - Main telemetry system
- `src/systems/ui/mod.rs` - Module integration
- `src/plugins/ui_plugin.rs` - Plugin registration
- `src/systems/ui/controls_ui.rs` - Controls guide update

#### System Architecture
- **Input System**: F4 key detection and toggle logic
- **Update System**: Real-time telemetry data processing
- **Hide System**: Automatic visibility management
- **Resource State**: Telemetry visibility and timing control

This telemetry dashboard brings the full Bugatti Chiron experience to life with professional-grade hypercar monitoring!
