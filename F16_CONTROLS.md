# F16 Fighter Jet - Realistic Flight Controls

## Changes Made

### Spawn Location
- **OLD**: F16 spawned at Vec3(80.0, 2.0, 120.0) - far from player
- **NEW**: F16 spawns at Vec3(5.0, 2.0, 0.0) - 5 units to the right of player spawn for immediate access

### Control System
Completely replaced arcade-style controls with realistic flight dynamics:

#### Flight Control Inputs
- **Pitch (Elevator)**: W/S keys or Arrow Up/Down
  - Nose up/down movement with gradual input response
  - Returns to neutral when released
  
- **Roll (Ailerons)**: A/D keys or Arrow Left/Right  
  - Banking left/right for turns
  - Returns to neutral when released
  
- **Yaw (Rudder)**: Q/E keys
  - Fine directional control
  - Less effective than pitch/roll (realistic)
  
- **Throttle**: Shift (increase) / Ctrl (decrease)
  - Gradual power adjustment from 0-100%
  - Engine spool-up time for realism
  
- **Afterburner**: Space key (hold)
  - 80% thrust boost when active
  - Drains more rapidly when released

### Realistic Flight Physics

#### Aerodynamic Forces
- **Thrust**: Applied along aircraft forward direction
- **Drag**: Quadratic increase with airspeed
- **Lift**: Dependent on airspeed and angle of attack
- **Gravity**: Constant downward force

#### Flight Characteristics
- **Stall Speed**: 40 units/sec minimum for lift generation
- **Max Speed**: 300 units/sec with safety limiting
- **Control Effectiveness**: Reduced at low airspeeds (realistic)
- **Stall Behavior**: Turbulence and loss of control below stall speed

#### Safety Features
- **Speed Limiting**: Prevents unsafe velocities
- **Collision Groups**: Proper physics interaction with world/player
- **Position Validation**: Safe spawn location with clearance
- **Control Input Clamping**: All inputs normalized to -1.0 to 1.0 range

## Technical Implementation

### New Components
- `AircraftFlight`: Comprehensive flight dynamics component
  - Control surface positions (pitch, roll, yaw, throttle)
  - Flight state variables (airspeed, angle of attack, etc.)
  - Aerodynamic properties (lift/drag coefficients)
  - Engine state (thrust, afterburner, spool time)

### Physics Integration
- Force-based movement system instead of direct velocity control
- Realistic rotational dynamics based on control input effectiveness
- Proper integration with Bevy's physics system (bevy_rapier3d)

### Performance Safeguards
- Velocity clamping prevents physics system instability
- Input validation ensures all values stay within safe ranges
- Gradual control response prevents jarring movements
