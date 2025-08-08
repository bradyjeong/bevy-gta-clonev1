# F16 Fighter Jet - Realistic Flight Controls

## Current Control Scheme (Updated)

### Primary Flight Controls
- **Pitch Control**: 
  - **W** = Pitch Up (nose up)
  - **S** = Pitch Down (nose down)
  
- **Roll Control**:
  - **A** = Roll Left (bank left)
  - **D** = Roll Right (bank right)
  
- **Yaw Control**:
  - **Q** = Yaw Left (rudder left)
  - **E** = Yaw Right (rudder right)

### Engine Controls  
- **Throttle** (Multiple Options):
  - **Arrow Up** = Increase Throttle
  - **Arrow Down** = Decrease Throttle  
  - **Left Shift** = Increase Throttle (alternative)
  - **Left Ctrl** = Decrease Throttle (alternative)
  
- **Afterburner**:
  - **Space** = Activate Afterburner (hold for continuous thrust)
  - 0.3 second fuel flow delay for realism
  - 35% additional thrust when active

### Utility Controls
- **F** = Exit F16 (return to player)
- **F1** = Debug Information
- **F2** = Emergency Reset

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
- Velocity-based movement system with unified physics approach
- Realistic rotational dynamics based on control input effectiveness
- Proper integration with Bevy's physics system (bevy_rapier3d)

### Performance Safeguards
- Velocity clamping prevents physics system instability
- Input validation ensures all values stay within safe ranges
- Gradual control response prevents jarring movements
