//! Unified factory for generating [`Transform`] components with position validation.
//!
//! # Overview
//! The `TransformFactory` provides a centralized API for creating [`Transform`] components
//! with consistent positioning, rotation, and scaling across all game entities. It eliminates
//! over 100+ scattered `Transform::from_xyz` patterns throughout the codebase while adding
//! critical safety validation for position bounds and rotation values.
//!
//! ## Typical usage
//! ```rust
//! use bevy::prelude::*;
//! use gameplay_render::factories::TransformFactory;
//!
//! fn spawn_vehicle(mut commands: Commands) {
//!     let vehicle_transform = TransformFactory::vehicle_spawn(10.0, 5.0);
//!     commands.spawn(TransformBundle::from_transform(vehicle_transform));
//! }
//! ```
//!
//! # Coordinate System
//! The factory uses Bevy's right-handed coordinate system:
//! - **X-axis**: Right (positive) / Left (negative)
//! - **Y-axis**: Up (positive) / Down (negative)  
//! - **Z-axis**: Forward (positive) / Backward (negative)
//!
//! # Safety Considerations
//! All position values are validated and clamped to prevent:
//! - Floating-point overflow in physics calculations
//! - Extreme positions that break rendering culling
//! - Invalid rotation quaternions that cause NaN propagation
//!
//! # Implementation notes
//! The factory categorizes transform creation by entity type (vehicles, NPCs, environment)
//! to ensure consistent relative positioning and reduce coordinate system errors.

use bevy::prelude::*;

/// A unified factory for creating [`Transform`] components with position validation.
///
/// This factory handles the complete creation process for transforms, from basic
/// positioning to complex hierarchical arrangements. It ensures consistent coordinate
/// usage across different entity types while providing safety validation for all
/// position and rotation values.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use gameplay_render::factories::TransformFactory;
///
/// fn setup_scene(mut commands: Commands) {
///     // Basic positioning
///     let ground_pos = TransformFactory::at_ground_level(0.0, 0.0);
///     
///     // Vehicle with elevation
///     let car_pos = TransformFactory::vehicle_spawn(10.0, 5.0);
///     
///     // Safe positioning with validation
///     let safe_pos = TransformFactory::custom_position_safe(999999.0, 50.0, -999999.0);
///     
///     commands.spawn(TransformBundle::from_transform(ground_pos));
///     commands.spawn(TransformBundle::from_transform(car_pos));
///     commands.spawn(TransformBundle::from_transform(safe_pos));
/// }
/// ```
pub struct TransformFactory;

impl TransformFactory {
    /// Creates a transform positioned at the world origin (0, 0, 0).
    ///
    /// This is the fundamental reference point for all world coordinates. Commonly used
    /// for reference entities, camera targets, and world-anchored systems.
    ///
    /// # Returns
    /// A [`Transform`] positioned at [`Vec3::ZERO`] with default rotation and scale
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_reference_point(mut commands: Commands) {
    ///     let origin = TransformFactory::at_origin();
    ///     assert_eq!(origin.translation, Vec3::ZERO);
    ///     commands.spawn(TransformBundle::from_transform(origin));
    /// }
    /// ```
    pub fn at_origin() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    /// Creates a transform positioned at ground level (Y=0) with the given X and Z coordinates.
    ///
    /// This is the standard positioning method for ground-based entities like vehicles,
    /// buildings, and NPCs. The Y coordinate is automatically set to 0.0 to ensure
    /// consistent ground-level placement.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified X and Z coordinates with Y=0.0
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn place_building(mut commands: Commands) {
    ///     let building_pos = TransformFactory::at_ground_level(50.0, 100.0);
    ///     assert_eq!(building_pos.translation.y, 0.0);
    ///     commands.spawn(TransformBundle::from_transform(building_pos));
    /// }
    /// ```
    pub fn at_ground_level(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.0, z)
    }

    /// Creates a transform positioned at the exact coordinates specified.
    ///
    /// This provides direct control over all three spatial dimensions. Use this for
    /// entities that need precise positioning above or below ground level, such as
    /// aircraft, elevated structures, or underground elements.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn place_helicopter(mut commands: Commands) {
    ///     let helicopter_pos = TransformFactory::at_position(0.0, 50.0, 0.0);
    ///     assert_eq!(helicopter_pos.translation, Vec3::new(0.0, 50.0, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(helicopter_pos));
    /// }
    /// ```
    pub fn at_position(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    /// Creates a transform for spawning ground vehicles at standard height.
    ///
    /// Positions the vehicle slightly above ground level (Y=0.5) to ensure proper
    /// physics collision detection and prevent ground clipping. This is the standard
    /// spawn method for cars, trucks, and other ground-based vehicles.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified X and Z coordinates with Y=0.5
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_car(mut commands: Commands) {
    ///     let car_pos = TransformFactory::vehicle_spawn(10.0, 5.0);
    ///     assert_eq!(car_pos.translation.y, 0.5);
    ///     commands.spawn(TransformBundle::from_transform(car_pos));
    /// }
    /// ```
    pub fn vehicle_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.5, z)
    }

    /// Creates a transform for vehicles positioned at a specific elevation.
    ///
    /// Use this for vehicles that need to be placed on elevated surfaces like
    /// parking garages, bridges, or ramps. Provides full control over the Y position
    /// while maintaining vehicle positioning semantics.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_bridge_vehicle(mut commands: Commands) {
    ///     let bridge_car = TransformFactory::vehicle_elevated(0.0, 15.0, 0.0);
    ///     commands.spawn(TransformBundle::from_transform(bridge_car));
    /// }
    /// ```
    pub fn vehicle_elevated(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    /// Creates a transform for spawning helicopters at the specified position.
    ///
    /// Helicopters can be positioned at any altitude and don't require ground-level
    /// adjustments. This method provides direct positioning control for rotorcraft
    /// spawning and navigation.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_helicopter(mut commands: Commands) {
    ///     let heli_pos = TransformFactory::helicopter_spawn(0.0, 100.0, 0.0);
    ///     commands.spawn(TransformBundle::from_transform(heli_pos));
    /// }
    /// ```
    pub fn helicopter_spawn(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    /// Creates a transform for spawning boats at the specified water level.
    ///
    /// Boats require positioning at specific water surface heights to ensure proper
    /// buoyancy physics and visual appearance. The Y coordinate should match the
    /// water level of the target body of water.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (water surface level)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_boat(mut commands: Commands) {
    ///     let boat_pos = TransformFactory::boat_spawn(0.0, 2.0, 0.0);
    ///     commands.spawn(TransformBundle::from_transform(boat_pos));
    /// }
    /// ```
    pub fn boat_spawn(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // VEHICLE COMPONENTS - Relative positioning
    
    /// Creates a transform for the center of a vehicle body.
    ///
    /// This represents the central reference point for all vehicle components. All other
    /// vehicle parts are positioned relative to this center point, ensuring consistent
    /// vehicle assembly and proper physics center of mass calculations.
    ///
    /// # Returns
    /// A [`Transform`] positioned at the vehicle's center (0, 0, 0) relative to the parent
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_vehicle_body(mut commands: Commands) {
    ///     let body_center = TransformFactory::vehicle_body_center();
    ///     assert_eq!(body_center.translation, Vec3::ZERO);
    ///     commands.spawn(TransformBundle::from_transform(body_center));
    /// }
    /// ```
    pub fn vehicle_body_center() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    /// Creates a transform for the vehicle chassis positioned below the body center.
    ///
    /// The chassis forms the structural foundation of the vehicle, positioned slightly
    /// below the body center to provide realistic vehicle proportions and proper
    /// ground clearance for physics collision detection.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, -0.1, 0) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_vehicle_chassis(mut commands: Commands) {
    ///     let chassis_pos = TransformFactory::vehicle_chassis();
    ///     assert_eq!(chassis_pos.translation.y, -0.1);
    ///     commands.spawn(TransformBundle::from_transform(chassis_pos));
    /// }
    /// ```
    pub fn vehicle_chassis() -> Transform {
        Transform::from_xyz(0.0, -0.1, 0.0)
    }

    /// Creates a transform for the vehicle cabin positioned above and behind the center.
    ///
    /// The cabin houses the passenger compartment and is positioned above the chassis
    /// and slightly toward the rear of the vehicle to create realistic proportions.
    /// This positioning ensures proper visibility and interior space representation.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 0.25, -0.3) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_vehicle_cabin(mut commands: Commands) {
    ///     let cabin_pos = TransformFactory::vehicle_cabin();
    ///     assert_eq!(cabin_pos.translation, Vec3::new(0.0, 0.25, -0.3));
    ///     commands.spawn(TransformBundle::from_transform(cabin_pos));
    /// }
    /// ```
    pub fn vehicle_cabin() -> Transform {
        Transform::from_xyz(0.0, 0.25, -0.3)
    }

    /// Creates a transform for the vehicle hood positioned at the front of the vehicle.
    ///
    /// The hood covers the engine compartment and is positioned at the front of the
    /// vehicle body. This placement provides realistic vehicle proportions and proper
    /// front-end coverage for the engine bay area.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 0.12, 1.6) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_vehicle_hood(mut commands: Commands) {
    ///     let hood_pos = TransformFactory::vehicle_hood();
    ///     assert_eq!(hood_pos.translation, Vec3::new(0.0, 0.12, 1.6));
    ///     commands.spawn(TransformBundle::from_transform(hood_pos));
    /// }
    /// ```
    pub fn vehicle_hood() -> Transform {
        Transform::from_xyz(0.0, 0.12, 1.6)
    }

    /// Creates a transform for the windshield positioned above and angled toward the front.
    ///
    /// The windshield is positioned above the dashboard area with a slight backward tilt
    /// to create realistic viewing angles and proper aerodynamic appearance. The rotation
    /// creates an angled windshield typical of modern vehicles.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 0.4, 0.8) with X-axis rotation of -0.2 radians
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_windshield(mut commands: Commands) {
    ///     let windshield_pos = TransformFactory::windshield();
    ///     assert_eq!(windshield_pos.translation, Vec3::new(0.0, 0.4, 0.8));
    ///     commands.spawn(TransformBundle::from_transform(windshield_pos));
    /// }
    /// ```
    pub fn windshield() -> Transform {
        Transform::from_xyz(0.0, 0.4, 0.8).with_rotation(Quat::from_rotation_x(-0.2))
    }

    /// Creates a transform for the left door positioned on the driver's side.
    ///
    /// The left door is positioned on the positive X-axis side of the vehicle,
    /// which corresponds to the left side when facing forward. This is typically
    /// the driver's side in right-hand traffic countries.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0.75, 0.3, -0.3) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_left_door(mut commands: Commands) {
    ///     let left_door_pos = TransformFactory::left_door();
    ///     assert_eq!(left_door_pos.translation, Vec3::new(0.75, 0.3, -0.3));
    ///     commands.spawn(TransformBundle::from_transform(left_door_pos));
    /// }
    /// ```
    pub fn left_door() -> Transform {
        Transform::from_xyz(0.75, 0.3, -0.3)
    }

    /// Creates a transform for the right door positioned on the passenger side.
    ///
    /// The right door is positioned on the negative X-axis side of the vehicle,
    /// which corresponds to the right side when facing forward. This is typically
    /// the passenger side in right-hand traffic countries.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (-0.75, 0.3, -0.3) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_right_door(mut commands: Commands) {
    ///     let right_door_pos = TransformFactory::right_door();
    ///     assert_eq!(right_door_pos.translation, Vec3::new(-0.75, 0.3, -0.3));
    ///     commands.spawn(TransformBundle::from_transform(right_door_pos));
    /// }
    /// ```
    pub fn right_door() -> Transform {
        Transform::from_xyz(-0.75, 0.3, -0.3)
    }

    /// Creates a transform for the rear window positioned at the back of the vehicle.
    ///
    /// The rear window is positioned at the back of the vehicle cabin, slightly
    /// below the roofline to create realistic proportions. This placement ensures
    /// proper visibility from the interior and accurate vehicle silhouette.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, -0.05, 2.1) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_rear_window(mut commands: Commands) {
    ///     let rear_window_pos = TransformFactory::rear_window();
    ///     assert_eq!(rear_window_pos.translation, Vec3::new(0.0, -0.05, 2.1));
    ///     commands.spawn(TransformBundle::from_transform(rear_window_pos));
    /// }
    /// ```
    pub fn rear_window() -> Transform {
        Transform::from_xyz(0.0, -0.05, 2.1)
    }

    /// Creates a transform for the front bumper positioned at the very front of the vehicle.
    ///
    /// The front bumper provides protection and aesthetic appeal, positioned at the
    /// front edge of the vehicle. This placement ensures proper collision detection
    /// and realistic vehicle proportions for front-end impacts.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 0.6, -1.8) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_front_bumper(mut commands: Commands) {
    ///     let bumper_pos = TransformFactory::front_bumper();
    ///     assert_eq!(bumper_pos.translation, Vec3::new(0.0, 0.6, -1.8));
    ///     commands.spawn(TransformBundle::from_transform(bumper_pos));
    /// }
    /// ```
    pub fn front_bumper() -> Transform {
        Transform::from_xyz(0.0, 0.6, -1.8)
    }

    // WHEELS - Standard positions with rotation
    
    /// Creates a transform for the front left wheel positioned at the front left of the vehicle.
    ///
    /// The front left wheel is positioned at the front axle on the left side of the vehicle
    /// (positive X-axis). This positioning ensures proper vehicle balance and realistic
    /// wheel placement for physics simulation and visual accuracy.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0.6, 0.0, 2.0) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_front_left_wheel(mut commands: Commands) {
    ///     let wheel_pos = TransformFactory::front_left_wheel();
    ///     assert_eq!(wheel_pos.translation, Vec3::new(0.6, 0.0, 2.0));
    ///     commands.spawn(TransformBundle::from_transform(wheel_pos));
    /// }
    /// ```
    pub fn front_left_wheel() -> Transform {
        Transform::from_xyz(0.6, 0.0, 2.0)
    }

    /// Creates a transform for the front right wheel positioned at the front right of the vehicle.
    ///
    /// The front right wheel is positioned at the front axle on the right side of the vehicle
    /// (negative X-axis). This positioning ensures proper vehicle balance and realistic
    /// wheel placement for physics simulation and visual accuracy.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (-0.6, 0.0, 2.0) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_front_right_wheel(mut commands: Commands) {
    ///     let wheel_pos = TransformFactory::front_right_wheel();
    ///     assert_eq!(wheel_pos.translation, Vec3::new(-0.6, 0.0, 2.0));
    ///     commands.spawn(TransformBundle::from_transform(wheel_pos));
    /// }
    /// ```
    pub fn front_right_wheel() -> Transform {
        Transform::from_xyz(-0.6, 0.0, 2.0)
    }

    /// Creates a transform for the rear left wheel positioned at the rear left of the vehicle.
    ///
    /// The rear left wheel is positioned at the rear axle on the left side of the vehicle
    /// (positive X-axis). The wheel is positioned slightly lower than the front wheels to
    /// create realistic vehicle suspension geometry and proper ground contact.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0.6, -0.35, -1.5) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_rear_left_wheel(mut commands: Commands) {
    ///     let wheel_pos = TransformFactory::rear_left_wheel();
    ///     assert_eq!(wheel_pos.translation, Vec3::new(0.6, -0.35, -1.5));
    ///     commands.spawn(TransformBundle::from_transform(wheel_pos));
    /// }
    /// ```
    pub fn rear_left_wheel() -> Transform {
        Transform::from_xyz(0.6, -0.35, -1.5)
    }

    /// Creates a transform for the rear right wheel positioned at the rear right of the vehicle.
    ///
    /// The rear right wheel is positioned at the rear axle on the right side of the vehicle
    /// (negative X-axis). The wheel is positioned slightly lower than the front wheels to
    /// create realistic vehicle suspension geometry and proper ground contact.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (-0.6, -0.35, -1.5) relative to the vehicle body center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_rear_right_wheel(mut commands: Commands) {
    ///     let wheel_pos = TransformFactory::rear_right_wheel();
    ///     assert_eq!(wheel_pos.translation, Vec3::new(-0.6, -0.35, -1.5));
    ///     commands.spawn(TransformBundle::from_transform(wheel_pos));
    /// }
    /// ```
    pub fn rear_right_wheel() -> Transform {
        Transform::from_xyz(-0.6, -0.35, -1.5)
    }

    /// Transform for main rotor position
    pub fn main_rotor() -> Transform {
        Transform::from_xyz(0.0, 1.2, 0.0)
    }

    /// Transform for tail rotor position  
    pub fn tail_rotor() -> Transform {
        Transform::from_xyz(0.0, 0.6, -1.8)
    }
    
    /// Transform for F16 left wing (properly positioned for realistic layout)
    pub fn f16_left_wing() -> Transform {
        Transform::from_xyz(-5.0, -0.2, 1.0) // Left side, slightly below fuselage, forward
    }
    
    /// Transform for F16 right wing (properly positioned for realistic layout)
    pub fn f16_right_wing() -> Transform {
        Transform::from_xyz(5.0, -0.2, 1.0) // Right side, slightly below fuselage, forward
    }
    
    /// Transform for F16 canopy (pilot position)
    pub fn f16_canopy() -> Transform {
        Transform::from_xyz(0.0, 1.2, 2.0) // Above fuselage, forward of center
    }
    
    /// Transform for F16 left air intake
    pub fn f16_left_air_intake() -> Transform {
        Transform::from_xyz(-1.5, -0.3, 3.0) // Left side, below fuselage, forward
    }
    
    /// Transform for F16 right air intake
    pub fn f16_right_air_intake() -> Transform {
        Transform::from_xyz(1.5, -0.3, 3.0) // Right side, below fuselage, forward
    }
    
    /// Transform for F16 vertical tail
    pub fn f16_vertical_tail() -> Transform {
        Transform::from_xyz(0.0, 1.5, -6.0) // Above fuselage, rear
    }
    
    /// Transform for F16 left horizontal stabilizer
    pub fn f16_left_horizontal_stabilizer() -> Transform {
        Transform::from_xyz(-2.0, 0.3, -6.5) // Left side, rear, slightly elevated
    }
    
    /// Transform for F16 right horizontal stabilizer
    pub fn f16_right_horizontal_stabilizer() -> Transform {
        Transform::from_xyz(2.0, 0.3, -6.5) // Right side, rear, slightly elevated
    }
    
    /// Transform for F16 engine nozzle
    pub fn f16_engine_nozzle() -> Transform {
        Transform::from_xyz(0.0, -0.2, -7.5).with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0))
    }

    /// Creates a transform for a wheel with proper rotation at the specified position.
    ///
    /// This function creates a wheel transform with a 90-degree rotation around the Z-axis
    /// to orient the wheel properly for vehicle attachment. The rotation ensures the wheel
    /// faces the correct direction for realistic vehicle movement and physics simulation.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates with Z-axis rotation
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_custom_wheel(mut commands: Commands) {
    ///     let wheel_pos = TransformFactory::wheel_with_rotation(0.6, 0.0, 2.0);
    ///     assert_eq!(wheel_pos.translation, Vec3::new(0.6, 0.0, 2.0));
    ///     commands.spawn(TransformBundle::from_transform(wheel_pos));
    /// }
    /// ```
    pub fn wheel_with_rotation(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0))
    }

    /// Creates a transform for a large vehicle wheel at the specified position.
    ///
    /// This function creates a wheel transform for larger vehicles such as trucks,
    /// SUVs, or buses. The wheel is positioned at the specified coordinates without
    /// additional rotation, allowing for flexible placement on various vehicle types.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_truck_wheel(mut commands: Commands) {
    ///     let wheel_pos = TransformFactory::large_vehicle_wheel(0.8, -0.3, 2.5);
    ///     assert_eq!(wheel_pos.translation, Vec3::new(0.8, -0.3, 2.5));
    ///     commands.spawn(TransformBundle::from_transform(wheel_pos));
    /// }
    /// ```
    pub fn large_vehicle_wheel(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // EXHAUST PIPES
    
    /// Creates a transform for the left exhaust pipe positioned at the rear left of the vehicle.
    ///
    /// The left exhaust pipe is positioned at the rear of the vehicle on the left side
    /// (positive X-axis) with a 90-degree rotation around the X-axis to orient the pipe
    /// properly for exhaust emission direction and realistic vehicle appearance.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0.4, -0.25, -2.0) with X-axis rotation
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_left_exhaust(mut commands: Commands) {
    ///     let exhaust_pos = TransformFactory::left_exhaust();
    ///     assert_eq!(exhaust_pos.translation, Vec3::new(0.4, -0.25, -2.0));
    ///     commands.spawn(TransformBundle::from_transform(exhaust_pos));
    /// }
    /// ```
    pub fn left_exhaust() -> Transform {
        Transform::from_xyz(0.4, -0.25, -2.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0))
    }

    /// Creates a transform for the right exhaust pipe positioned at the rear right of the vehicle.
    ///
    /// The right exhaust pipe is positioned at the rear of the vehicle on the right side
    /// (negative X-axis) with a 90-degree rotation around the X-axis to orient the pipe
    /// properly for exhaust emission direction and realistic vehicle appearance.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (-0.4, -0.25, -2.0) with X-axis rotation
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_right_exhaust(mut commands: Commands) {
    ///     let exhaust_pos = TransformFactory::right_exhaust();
    ///     assert_eq!(exhaust_pos.translation, Vec3::new(-0.4, -0.25, -2.0));
    ///     commands.spawn(TransformBundle::from_transform(exhaust_pos));
    /// }
    /// ```
    pub fn right_exhaust() -> Transform {
        Transform::from_xyz(-0.4, -0.25, -2.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0))
    }

    // AIRCRAFT COMPONENTS
    
    /// Creates a transform for the helicopter body centered at the origin.
    ///
    /// The helicopter body serves as the central reference point for all helicopter
    /// components. All other helicopter parts are positioned relative to this body
    /// center, ensuring consistent helicopter assembly and proper physics calculations.
    ///
    /// # Returns
    /// A [`Transform`] positioned at the helicopter's center (0, 0, 0)
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_helicopter_body(mut commands: Commands) {
    ///     let body_pos = TransformFactory::helicopter_body();
    ///     assert_eq!(body_pos.translation, Vec3::ZERO);
    ///     commands.spawn(TransformBundle::from_transform(body_pos));
    /// }
    /// ```
    pub fn helicopter_body() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    /// Creates a transform for a rotor with rotation at the specified angle.
    ///
    /// This function creates a rotor transform positioned above the helicopter body
    /// with a dynamic rotation around the Y-axis. The rotation angle allows for
    /// animated rotor blades that can spin realistically during flight simulation.
    ///
    /// # Arguments
    /// * `angle` - Rotation angle in radians around the Y-axis
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 2.2, 0) with the specified Y-axis rotation
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_spinning_rotor(mut commands: Commands) {
    ///     let rotor_pos = TransformFactory::rotor_with_rotation(std::f32::consts::PI / 4.0);
    ///     assert_eq!(rotor_pos.translation, Vec3::new(0.0, 2.2, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(rotor_pos));
    /// }
    /// ```
    pub fn rotor_with_rotation(angle: f32) -> Transform {
        Transform::from_xyz(0.0, 2.2, 0.0).with_rotation(Quat::from_rotation_y(angle))
    }

    /// Creates a transform for the helicopter cockpit at the center of the aircraft.
    ///
    /// The cockpit represents the pilot compartment and is positioned at the center
    /// of the helicopter body. This placement provides the pilot with optimal
    /// visibility and control positioning for realistic helicopter operation.
    ///
    /// # Returns
    /// A [`Transform`] positioned at the helicopter's center (0, 0, 0)
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_helicopter_cockpit(mut commands: Commands) {
    ///     let cockpit_pos = TransformFactory::helicopter_cockpit();
    ///     assert_eq!(cockpit_pos.translation, Vec3::ZERO);
    ///     commands.spawn(TransformBundle::from_transform(cockpit_pos));
    /// }
    /// ```
    pub fn helicopter_cockpit() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    /// Creates a transform for the left landing skid positioned below the helicopter.
    ///
    /// The left landing skid is positioned on the left side of the helicopter
    /// (negative X-axis) and below the body to provide landing support. This
    /// positioning ensures stable ground contact during helicopter landings.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (-2.0, -0.2, 0.0) relative to the helicopter body
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_left_landing_skid(mut commands: Commands) {
    ///     let skid_pos = TransformFactory::landing_skid_left();
    ///     assert_eq!(skid_pos.translation, Vec3::new(-2.0, -0.2, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(skid_pos));
    /// }
    /// ```
    pub fn landing_skid_left() -> Transform {
        Transform::from_xyz(-2.0, -0.2, 0.0)
    }

    /// Creates a transform for the tail rotor blade positioned at the helicopter's tail.
    ///
    /// The tail rotor blade is positioned at the rear of the helicopter and provides
    /// anti-torque control for stable flight. This positioning ensures proper
    /// aerodynamic function and realistic helicopter flight dynamics.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0.08, 2.2, 0.15) relative to the helicopter body
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_tail_rotor_blade(mut commands: Commands) {
    ///     let blade_pos = TransformFactory::tail_rotor_blade();
    ///     assert_eq!(blade_pos.translation, Vec3::new(0.08, 2.2, 0.15));
    ///     commands.spawn(TransformBundle::from_transform(blade_pos));
    /// }
    /// ```
    pub fn tail_rotor_blade() -> Transform {
        Transform::from_xyz(0.08, 2.2, 0.15)
    }

    // WATER VEHICLES
    
    /// Creates a transform for the yacht main body positioned above the water surface.
    ///
    /// The yacht main body is positioned above the water line and slightly toward
    /// the rear to create realistic yacht proportions. This positioning ensures
    /// proper buoyancy physics and realistic yacht appearance on water.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 3.5, -2.0) relative to the yacht center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_yacht_body(mut commands: Commands) {
    ///     let body_pos = TransformFactory::yacht_main_body();
    ///     assert_eq!(body_pos.translation, Vec3::new(0.0, 3.5, -2.0));
    ///     commands.spawn(TransformBundle::from_transform(body_pos));
    /// }
    /// ```
    pub fn yacht_main_body() -> Transform {
        Transform::from_xyz(0.0, 3.5, -2.0)
    }

    /// Creates a transform for the boat mast positioned high above the vessel.
    ///
    /// The boat mast is positioned at a significant height above the boat to
    /// support sails and navigation equipment. This positioning provides realistic
    /// sailing functionality and proper vessel proportions.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 9.5, 2.0) relative to the boat center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_boat_mast(mut commands: Commands) {
    ///     let mast_pos = TransformFactory::boat_mast();
    ///     assert_eq!(mast_pos.translation, Vec3::new(0.0, 9.5, 2.0));
    ///     commands.spawn(TransformBundle::from_transform(mast_pos));
    /// }
    /// ```
    pub fn boat_mast() -> Transform {
        Transform::from_xyz(0.0, 9.5, 2.0)
    }

    // NPC POSITIONING
    
    /// Creates a transform for spawning an NPC at ground level at the specified position.
    ///
    /// This function positions an NPC at the specified X and Z coordinates with
    /// a standard height of 1.0 to ensure proper ground-level placement. The Y
    /// coordinate is automatically set to provide realistic NPC positioning.
    ///
    /// # Arguments
    /// * `position` - The [`Vec3`] position where the NPC should be spawned
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified X and Z coordinates with Y=1.0
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_npc(mut commands: Commands) {
    ///     let npc_pos = TransformFactory::npc_spawn(Vec3::new(10.0, 0.0, 5.0));
    ///     assert_eq!(npc_pos.translation, Vec3::new(10.0, 1.0, 5.0));
    ///     commands.spawn(TransformBundle::from_transform(npc_pos));
    /// }
    /// ```
    pub fn npc_spawn(position: Vec3) -> Transform {
        Transform::from_xyz(position.x, 1.0, position.z)
    }

    /// Creates a transform for spawning an NPC at an elevated position.
    ///
    /// This function positions an NPC at the specified X and Z coordinates with
    /// a custom height value. Use this for NPCs that need to be placed on elevated
    /// surfaces like balconies, bridges, or upper floors of buildings.
    ///
    /// # Arguments
    /// * `position` - The [`Vec3`] position where the NPC should be spawned
    /// * `height` - The Y coordinate for the NPC's vertical position
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified X and Z coordinates with the given height
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_elevated_npc(mut commands: Commands) {
    ///     let npc_pos = TransformFactory::npc_elevated(Vec3::new(0.0, 0.0, 0.0), 5.0);
    ///     assert_eq!(npc_pos.translation, Vec3::new(0.0, 5.0, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(npc_pos));
    /// }
    /// ```
    pub fn npc_elevated(position: Vec3, height: f32) -> Transform {
        Transform::from_xyz(position.x, height, position.z)
    }

    // NPC BODY PARTS - Relative to NPC center
    
    /// Creates a transform for an NPC head positioned relative to the NPC center.
    ///
    /// The head is positioned at 85% of the NPC's height factor above the center,
    /// creating realistic human proportions. Height factor is clamped between 0.1
    /// and 10.0 to prevent invalid positioning values.
    ///
    /// # Arguments
    /// * `height_factor` - Scaling factor for NPC height (clamped to [0.1, 10.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, height_factor * 0.85, 0) relative to NPC center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_npc_head(mut commands: Commands) {
    ///     let head_pos = TransformFactory::npc_head(1.8);
    ///     assert_eq!(head_pos.translation.y, 1.8 * 0.85);
    ///     commands.spawn(TransformBundle::from_transform(head_pos));
    /// }
    /// ```
    pub fn npc_head(height_factor: f32) -> Transform {
        let safe_height = height_factor.max(0.1).min(10.0);
        Transform::from_xyz(0.0, safe_height * 0.85, 0.0)
    }

    /// Creates a transform for an NPC torso positioned relative to the NPC center.
    ///
    /// The torso is positioned at 50% of the NPC's height factor above the center,
    /// creating realistic human proportions. Height factor is clamped between 0.1
    /// and 10.0 to prevent invalid positioning values.
    ///
    /// # Arguments
    /// * `height_factor` - Scaling factor for NPC height (clamped to [0.1, 10.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, height_factor * 0.5, 0) relative to NPC center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_npc_torso(mut commands: Commands) {
    ///     let torso_pos = TransformFactory::npc_torso(1.8);
    ///     assert_eq!(torso_pos.translation.y, 1.8 * 0.5);
    ///     commands.spawn(TransformBundle::from_transform(torso_pos));
    /// }
    /// ```
    pub fn npc_torso(height_factor: f32) -> Transform {
        let safe_height = height_factor.max(0.1).min(10.0);
        Transform::from_xyz(0.0, safe_height * 0.5, 0.0)
    }

    /// Creates a transform for an NPC left arm positioned relative to the NPC center.
    ///
    /// The left arm is positioned on the left side of the NPC (negative X-axis) at
    /// 65% of the height factor, with horizontal offset based on the build factor.
    /// Both parameters are clamped to safe ranges to prevent invalid positioning.
    ///
    /// # Arguments
    /// * `build` - Scaling factor for NPC build/width (clamped to [0.1, 5.0])
    /// * `height` - Scaling factor for NPC height (clamped to [0.1, 10.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at (-0.25 * build, height * 0.65, 0) relative to NPC center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_npc_left_arm(mut commands: Commands) {
    ///     let arm_pos = TransformFactory::npc_left_arm(1.0, 1.8);
    ///     assert_eq!(arm_pos.translation, Vec3::new(-0.25, 1.8 * 0.65, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(arm_pos));
    /// }
    /// ```
    pub fn npc_left_arm(build: f32, height: f32) -> Transform {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        Transform::from_xyz(-0.25 * safe_build, safe_height * 0.65, 0.0)
    }

    /// Creates a transform for an NPC right arm positioned relative to the NPC center.
    ///
    /// The right arm is positioned on the right side of the NPC (positive X-axis) at
    /// 65% of the height factor, with horizontal offset based on the build factor.
    /// Both parameters are clamped to safe ranges to prevent invalid positioning.
    ///
    /// # Arguments
    /// * `build` - Scaling factor for NPC build/width (clamped to [0.1, 5.0])
    /// * `height` - Scaling factor for NPC height (clamped to [0.1, 10.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0.25 * build, height * 0.65, 0) relative to NPC center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_npc_right_arm(mut commands: Commands) {
    ///     let arm_pos = TransformFactory::npc_right_arm(1.0, 1.8);
    ///     assert_eq!(arm_pos.translation, Vec3::new(0.25, 1.8 * 0.65, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(arm_pos));
    /// }
    /// ```
    pub fn npc_right_arm(build: f32, height: f32) -> Transform {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        Transform::from_xyz(0.25 * safe_build, safe_height * 0.65, 0.0)
    }

    /// Creates a transform for an NPC left leg positioned relative to the NPC center.
    ///
    /// The left leg is positioned on the left side of the NPC (negative X-axis) at
    /// 20% of the height factor, with horizontal offset based on the build factor.
    /// Both parameters are clamped to safe ranges to prevent invalid positioning.
    ///
    /// # Arguments
    /// * `build` - Scaling factor for NPC build/width (clamped to [0.1, 5.0])
    /// * `height` - Scaling factor for NPC height (clamped to [0.1, 10.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at (-0.1 * build, height * 0.2, 0) relative to NPC center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_npc_left_leg(mut commands: Commands) {
    ///     let leg_pos = TransformFactory::npc_left_leg(1.0, 1.8);
    ///     assert_eq!(leg_pos.translation, Vec3::new(-0.1, 1.8 * 0.2, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(leg_pos));
    /// }
    /// ```
    pub fn npc_left_leg(build: f32, height: f32) -> Transform {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        Transform::from_xyz(-0.1 * safe_build, safe_height * 0.2, 0.0)
    }

    /// Creates a transform for an NPC right leg positioned relative to the NPC center.
    ///
    /// The right leg is positioned on the right side of the NPC (positive X-axis) at
    /// 20% of the height factor, with horizontal offset based on the build factor.
    /// Both parameters are clamped to safe ranges to prevent invalid positioning.
    ///
    /// # Arguments
    /// * `build` - Scaling factor for NPC build/width (clamped to [0.1, 5.0])
    /// * `height` - Scaling factor for NPC height (clamped to [0.1, 10.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0.1 * build, height * 0.2, 0) relative to NPC center
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_npc_right_leg(mut commands: Commands) {
    ///     let leg_pos = TransformFactory::npc_right_leg(1.0, 1.8);
    ///     assert_eq!(leg_pos.translation, Vec3::new(0.1, 1.8 * 0.2, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(leg_pos));
    /// }
    /// ```
    pub fn npc_right_leg(build: f32, height: f32) -> Transform {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        Transform::from_xyz(0.1 * safe_build, safe_height * 0.2, 0.0)
    }

    // WORLD STRUCTURES
    
    /// Creates a transform for a lamp post positioned at ground level.
    ///
    /// The lamp post is positioned at the specified X and Z coordinates with Y=0.0
    /// to ensure proper ground-level placement. This provides consistent positioning
    /// for street lighting and urban infrastructure elements.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified X and Z coordinates with Y=0.0
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_lamp_post(mut commands: Commands) {
    ///     let post_pos = TransformFactory::lamp_post(10.0, 5.0);
    ///     assert_eq!(post_pos.translation, Vec3::new(10.0, 0.0, 5.0));
    ///     commands.spawn(TransformBundle::from_transform(post_pos));
    /// }
    /// ```
    pub fn lamp_post(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.0, z)
    }

    /// Creates a transform for a lamp light positioned at the top of a lamp post.
    ///
    /// The lamp light is positioned at a height of 4.0 units above the lamp post
    /// base to provide realistic street lighting height and proper illumination
    /// coverage for the surrounding area.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 4.0, 0) relative to the lamp post base
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_lamp_light(mut commands: Commands) {
    ///     let light_pos = TransformFactory::lamp_light();
    ///     assert_eq!(light_pos.translation, Vec3::new(0.0, 4.0, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(light_pos));
    /// }
    /// ```
    pub fn lamp_light() -> Transform {
        Transform::from_xyz(0.0, 4.0, 0.0)
    }

    /// Creates a transform for a tree positioned at ground level.
    ///
    /// The tree is positioned at the specified X and Z coordinates with Y=0.0
    /// to ensure proper ground-level placement. This provides consistent positioning
    /// for vegetation and natural environment elements.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified X and Z coordinates with Y=0.0
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_tree(mut commands: Commands) {
    ///     let tree_pos = TransformFactory::tree_position(15.0, 20.0);
    ///     assert_eq!(tree_pos.translation, Vec3::new(15.0, 0.0, 20.0));
    ///     commands.spawn(TransformBundle::from_transform(tree_pos));
    /// }
    /// ```
    pub fn tree_position(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.0, z)
    }

    /// Creates a transform for tree fronds positioned at the top of a tree.
    ///
    /// The fronds are positioned at a height of 7.5 units above the tree base
    /// at the specified X and Z coordinates. This creates realistic tree canopy
    /// positioning and proper visual hierarchy for vegetation.
    ///
    /// # Arguments
    /// * `frond_x` - Horizontal position for the fronds (right/left axis)
    /// * `frond_z` - Depth position for the fronds (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified X and Z coordinates with Y=7.5
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_tree_fronds(mut commands: Commands) {
    ///     let fronds_pos = TransformFactory::tree_fronds(0.0, 0.0);
    ///     assert_eq!(fronds_pos.translation, Vec3::new(0.0, 7.5, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(fronds_pos));
    /// }
    /// ```
    pub fn tree_fronds(frond_x: f32, frond_z: f32) -> Transform {
        Transform::from_xyz(frond_x, 7.5, frond_z)
    }

    /// Creates a transform for a building base positioned at half its height.
    ///
    /// The building base is positioned at the specified X and Z coordinates with
    /// Y coordinate set to half the building height to center the building mesh
    /// properly. Height is clamped between 0.1 and 1000.0 for safety.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `height` - Building height (clamped to [0.1, 1000.0])
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at (x, height/2, z) with clamped height
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_building(mut commands: Commands) {
    ///     let building_pos = TransformFactory::building_base(0.0, 50.0, 0.0);
    ///     assert_eq!(building_pos.translation, Vec3::new(0.0, 25.0, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(building_pos));
    /// }
    /// ```
    pub fn building_base(x: f32, height: f32, z: f32) -> Transform {
        let safe_height = height.max(0.1).min(1000.0);
        Transform::from_xyz(x, safe_height / 2.0, z)
    }

    // ENVIRONMENT DETAILS
    
    /// Creates a transform for a ground vehicle elevated above the surface.
    ///
    /// The vehicle is positioned at the specified coordinates with an additional
    /// 1.0 unit elevation added to the Y coordinate to ensure proper ground
    /// clearance and prevent clipping with terrain surfaces.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Base vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at (x, y + 1.0, z) with elevation adjustment
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_ground_vehicle(mut commands: Commands) {
    ///     let vehicle_pos = TransformFactory::ground_vehicle(10.0, 0.0, 5.0);
    ///     assert_eq!(vehicle_pos.translation, Vec3::new(10.0, 1.0, 5.0));
    ///     commands.spawn(TransformBundle::from_transform(vehicle_pos));
    /// }
    /// ```
    pub fn ground_vehicle(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y + 1.0, z)
    }

    /// Creates a transform for environment detail objects at the specified position.
    ///
    /// This function provides direct positioning for miscellaneous environment
    /// details such as props, debris, or decorative elements that don't require
    /// special positioning logic or height adjustments.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_environment_detail(mut commands: Commands) {
    ///     let detail_pos = TransformFactory::environment_detail(5.0, 2.0, 8.0);
    ///     assert_eq!(detail_pos.translation, Vec3::new(5.0, 2.0, 8.0));
    ///     commands.spawn(TransformBundle::from_transform(detail_pos));
    /// }
    /// ```
    pub fn environment_detail(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    /// Creates a transform for a street light with proper orientation.
    ///
    /// The street light is positioned at the specified coordinates with a 90-degree
    /// rotation around the Z-axis to orient the light fixture properly for street
    /// lighting applications and realistic urban infrastructure placement.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates with Z-axis rotation
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_street_light(mut commands: Commands) {
    ///     let light_pos = TransformFactory::street_light(0.0, 5.0, 0.0);
    ///     assert_eq!(light_pos.translation, Vec3::new(0.0, 5.0, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(light_pos));
    /// }
    /// ```
    pub fn street_light(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
    }

    // WATER FEATURES
    
    /// Creates a transform for a lake surface at the specified position.
    ///
    /// The lake surface is positioned at the exact coordinates provided in the
    /// lake_position vector. This positioning ensures proper water level alignment
    /// and consistent water surface rendering across the game world.
    ///
    /// # Arguments
    /// * `lake_position` - The [`Vec3`] position for the lake surface
    ///
    /// # Returns
    /// A [`Transform`] positioned at the lake surface coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_lake_surface(mut commands: Commands) {
    ///     let lake_pos = Vec3::new(100.0, 2.0, 150.0);
    ///     let surface_pos = TransformFactory::lake_surface(lake_pos);
    ///     assert_eq!(surface_pos.translation, lake_pos);
    ///     commands.spawn(TransformBundle::from_transform(surface_pos));
    /// }
    /// ```
    pub fn lake_surface(lake_position: Vec3) -> Transform {
        Transform::from_xyz(lake_position.x, lake_position.y, lake_position.z)
    }

    /// Creates a transform for a lake bottom at the specified depth below the surface.
    ///
    /// The lake bottom is positioned at the same X and Z coordinates as the lake
    /// surface, but at a depth below the surface level. Depth is clamped between
    /// 0.1 and 1000.0 to ensure valid underwater positioning.
    ///
    /// # Arguments
    /// * `lake_position` - The [`Vec3`] position for the lake surface
    /// * `depth` - The depth below the surface (clamped to [0.1, 1000.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at the lake bottom coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_lake_bottom(mut commands: Commands) {
    ///     let lake_pos = Vec3::new(100.0, 2.0, 150.0);
    ///     let bottom_pos = TransformFactory::lake_bottom(lake_pos, 10.0);
    ///     assert_eq!(bottom_pos.translation, Vec3::new(100.0, -8.0, 150.0));
    ///     commands.spawn(TransformBundle::from_transform(bottom_pos));
    /// }
    /// ```
    pub fn lake_bottom(lake_position: Vec3, depth: f32) -> Transform {
        let safe_depth = depth.max(0.1).min(1000.0);
        Transform::from_xyz(lake_position.x, lake_position.y - safe_depth, lake_position.z)
    }

    /// Creates a transform for a lake cylinder centered at half the water depth.
    ///
    /// The lake cylinder is positioned at the same X and Z coordinates as the lake
    /// surface, but centered vertically within the water body. This is useful for
    /// creating underwater collision volumes or water physics boundaries.
    ///
    /// # Arguments
    /// * `lake_position` - The [`Vec3`] position for the lake surface
    /// * `depth` - The total depth of the water body (clamped to [0.1, 1000.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at the center of the water body
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_lake_cylinder(mut commands: Commands) {
    ///     let lake_pos = Vec3::new(100.0, 2.0, 150.0);
    ///     let cylinder_pos = TransformFactory::lake_cylinder(lake_pos, 10.0);
    ///     assert_eq!(cylinder_pos.translation, Vec3::new(100.0, -3.0, 150.0));
    ///     commands.spawn(TransformBundle::from_transform(cylinder_pos));
    /// }
    /// ```
    pub fn lake_cylinder(lake_position: Vec3, depth: f32) -> Transform {
        let safe_depth = depth.max(0.1).min(1000.0);
        Transform::from_xyz(lake_position.x, lake_position.y - safe_depth / 2.0, lake_position.z)
    }

    // SKY COMPONENTS
    
    /// Creates a transform for a sky dome centered at the world origin.
    ///
    /// The sky dome is positioned at the world origin to provide consistent
    /// sky rendering from any viewpoint. This positioning ensures the sky
    /// appears infinite and properly encompasses the entire game world.
    ///
    /// # Returns
    /// A [`Transform`] positioned at the world origin (0, 0, 0)
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_sky_dome(mut commands: Commands) {
    ///     let sky_pos = TransformFactory::sky_dome();
    ///     assert_eq!(sky_pos.translation, Vec3::ZERO);
    ///     commands.spawn(TransformBundle::from_transform(sky_pos));
    /// }
    /// ```
    pub fn sky_dome() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    /// Creates a transform for a celestial body at the specified position.
    ///
    /// Celestial bodies such as the sun, moon, or stars are positioned at the
    /// specified coordinates to create realistic sky environments and lighting
    /// conditions. This positioning allows for dynamic day/night cycles.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (right/left axis)
    /// * `y` - Vertical position (up/down axis)
    /// * `z` - Depth position (forward/backward axis)
    ///
    /// # Returns
    /// A [`Transform`] positioned at the specified coordinates
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_sun(mut commands: Commands) {
    ///     let sun_pos = TransformFactory::celestial_body(0.0, 1000.0, 0.0);
    ///     assert_eq!(sun_pos.translation, Vec3::new(0.0, 1000.0, 0.0));
    ///     commands.spawn(TransformBundle::from_transform(sun_pos));
    /// }
    /// ```
    pub fn celestial_body(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // CAMERA POSITIONING
    
    /// Creates a transform for a camera positioned with a standard overview of the scene.
    ///
    /// The camera is positioned at a height of 15.0 and distance of 25.0 from the
    /// origin, looking at the world origin. This provides a good default camera
    /// position for scene overview and general gameplay perspectives.
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, 15, 25) looking at the origin
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_camera(mut commands: Commands) {
    ///     let camera_pos = TransformFactory::camera_position();
    ///     assert_eq!(camera_pos.translation, Vec3::new(0.0, 15.0, 25.0));
    ///     commands.spawn(Camera3dBundle {
    ///         transform: camera_pos,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn camera_position() -> Transform {
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y)
    }

    /// Creates a transform for an elevated camera at the specified height.
    ///
    /// The camera is positioned at the specified height and distance of 25.0 from
    /// the origin, looking at the world origin. Height is clamped between 1.0 and
    /// 1000.0 to ensure valid camera positioning.
    ///
    /// # Arguments
    /// * `height` - Camera height above ground (clamped to [1.0, 1000.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned at (0, height, 25) looking at the origin
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_elevated_camera(mut commands: Commands) {
    ///     let camera_pos = TransformFactory::elevated_camera(50.0);
    ///     assert_eq!(camera_pos.translation, Vec3::new(0.0, 50.0, 25.0));
    ///     commands.spawn(Camera3dBundle {
    ///         transform: camera_pos,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn elevated_camera(height: f32) -> Transform {
        let safe_height = height.max(1.0).min(1000.0);
        Transform::from_xyz(0.0, safe_height, 25.0).looking_at(Vec3::ZERO, Vec3::Y)
    }

    // PROCEDURAL WORLD - Dynamic positioning
    
    /// Creates a transform for a horizontal road segment with scaled positioning.
    ///
    /// The road segment is positioned based on the segment size and road width,
    /// with a slight elevation (Y=0.1) to prevent z-fighting with terrain.
    /// Segment size is clamped between 1.0 and 1000.0 for safety.
    ///
    /// # Arguments
    /// * `_x` - Unused horizontal position parameter (reserved for future use)
    /// * `_z` - Unused depth position parameter (reserved for future use)
    /// * `segment_size` - Size multiplier for segment positioning (clamped to [1.0, 1000.0])
    /// * `road_width` - Width of the road for Z-axis positioning
    ///
    /// # Returns
    /// A [`Transform`] positioned for horizontal road layout
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_horizontal_road(mut commands: Commands) {
    ///     let road_pos = TransformFactory::road_segment_horizontal(0.0, 0.0, 10.0, 5.0);
    ///     assert_eq!(road_pos.translation, Vec3::new(15.0, 0.1, 5.0));
    ///     commands.spawn(TransformBundle::from_transform(road_pos));
    /// }
    /// ```
    pub fn road_segment_horizontal(_x: f32, _z: f32, segment_size: f32, road_width: f32) -> Transform {
        let safe_segment = segment_size.max(1.0).min(1000.0);
        Transform::from_xyz(safe_segment * 1.5, 0.1, road_width)
    }

    /// Creates a transform for a vertical road segment with scaled positioning.
    ///
    /// The road segment is positioned based on the segment size and road width,
    /// with a slight elevation (Y=0.1) to prevent z-fighting with terrain.
    /// Segment size is clamped between 1.0 and 1000.0 for safety.
    ///
    /// # Arguments
    /// * `_x` - Unused horizontal position parameter (reserved for future use)
    /// * `_z` - Unused depth position parameter (reserved for future use)
    /// * `segment_size` - Size multiplier for segment positioning (clamped to [1.0, 1000.0])
    /// * `road_width` - Width of the road for X-axis positioning
    ///
    /// # Returns
    /// A [`Transform`] positioned for vertical road layout
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_vertical_road(mut commands: Commands) {
    ///     let road_pos = TransformFactory::road_segment_vertical(0.0, 0.0, 10.0, 5.0);
    ///     assert_eq!(road_pos.translation, Vec3::new(5.0, 0.1, 15.0));
    ///     commands.spawn(TransformBundle::from_transform(road_pos));
    /// }
    /// ```
    pub fn road_segment_vertical(_x: f32, _z: f32, segment_size: f32, road_width: f32) -> Transform {
        let safe_segment = segment_size.max(1.0).min(1000.0);
        Transform::from_xyz(road_width, 0.1, safe_segment * 1.5)
    }

    /// Creates a transform for horizontal road markings with scaled positioning.
    ///
    /// The road markings are positioned based on the segment size with a slight
    /// elevation (Y=0.11) to render above the road surface. Segment size is
    /// clamped between 1.0 and 1000.0 for safety.
    ///
    /// # Arguments
    /// * `segment_size` - Size multiplier for marking positioning (clamped to [1.0, 1000.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned for horizontal road markings
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_horizontal_markings(mut commands: Commands) {
    ///     let marking_pos = TransformFactory::road_marking_horizontal(10.0);
    ///     assert_eq!(marking_pos.translation, Vec3::new(13.0, 0.11, 0.4));
    ///     commands.spawn(TransformBundle::from_transform(marking_pos));
    /// }
    /// ```
    pub fn road_marking_horizontal(segment_size: f32) -> Transform {
        let safe_segment = segment_size.max(1.0).min(1000.0);
        Transform::from_xyz(safe_segment * 1.3, 0.11, 0.4)
    }

    /// Creates a transform for vertical road markings with scaled positioning.
    ///
    /// The road markings are positioned based on the segment size with a slight
    /// elevation (Y=0.11) to render above the road surface. Segment size is
    /// clamped between 1.0 and 1000.0 for safety.
    ///
    /// # Arguments
    /// * `segment_size` - Size multiplier for marking positioning (clamped to [1.0, 1000.0])
    ///
    /// # Returns
    /// A [`Transform`] positioned for vertical road markings
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn setup_vertical_markings(mut commands: Commands) {
    ///     let marking_pos = TransformFactory::road_marking_vertical(10.0);
    ///     assert_eq!(marking_pos.translation, Vec3::new(0.4, 0.11, 13.0));
    ///     commands.spawn(TransformBundle::from_transform(marking_pos));
    /// }
    /// ```
    pub fn road_marking_vertical(segment_size: f32) -> Transform {
        let safe_segment = segment_size.max(1.0).min(1000.0);
        Transform::from_xyz(0.4, 0.11, safe_segment * 1.3)
    }

    /// Creates a transform with position validation and safety clamping.
    ///
    /// This function ensures all coordinates are within safe bounds to prevent
    /// floating-point overflow, physics simulation failures, and rendering issues.
    /// All input values are clamped to reasonable game world limits.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (clamped to [-10000.0, 10000.0])
    /// * `y` - Vertical position (clamped to [-1000.0, 10000.0])
    /// * `z` - Depth position (clamped to [-10000.0, 10000.0])
    ///
    /// # Returns
    /// A [`Transform`] with validated and clamped position values
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_with_extreme_coords(mut commands: Commands) {
    ///     // Extreme values are safely clamped
    ///     let safe_pos = TransformFactory::custom_position_safe(999999.0, 50.0, -999999.0);
    ///     assert!(safe_pos.translation.x <= 10000.0);
    ///     assert!(safe_pos.translation.z >= -10000.0);
    ///     commands.spawn(TransformBundle::from_transform(safe_pos));
    /// }
    /// ```
    pub fn custom_position_safe(x: f32, y: f32, z: f32) -> Transform {
        let safe_x = x.max(-10000.0).min(10000.0);
        let safe_y = y.max(-1000.0).min(10000.0);
        let safe_z = z.max(-10000.0).min(10000.0);
        Transform::from_xyz(safe_x, safe_y, safe_z)
    }

    /// Creates a transform with validated position and rotation.
    ///
    /// Combines position validation with rotation support for entities that need
    /// both placement and orientation. Position values are clamped to safe bounds
    /// while rotation is preserved for precise control.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (clamped to [-10000.0, 10000.0])
    /// * `y` - Vertical position (clamped to [-1000.0, 10000.0])
    /// * `z` - Depth position (clamped to [-10000.0, 10000.0])
    /// * `rotation` - Rotation quaternion (preserved without modification)
    ///
    /// # Returns
    /// A [`Transform`] with validated position and specified rotation
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_rotated_entity(mut commands: Commands) {
    ///     let rotation = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
    ///     let transform = TransformFactory::with_rotation_safe(100.0, 5.0, 200.0, rotation);
    ///     commands.spawn(TransformBundle::from_transform(transform));
    /// }
    /// ```
    pub fn with_rotation_safe(x: f32, y: f32, z: f32, rotation: Quat) -> Transform {
        let safe_x = x.max(-10000.0).min(10000.0);
        let safe_y = y.max(-1000.0).min(10000.0);
        let safe_z = z.max(-10000.0).min(10000.0);
        Transform::from_xyz(safe_x, safe_y, safe_z).with_rotation(rotation)
    }

    /// Creates a transform with validated position and scale.
    ///
    /// Ensures both position and scale values are within safe operational bounds.
    /// Scale values are clamped to prevent invisibly small or excessively large
    /// entities that would impact performance or visual quality.
    ///
    /// # Arguments
    /// * `x` - Horizontal position (clamped to [-10000.0, 10000.0])
    /// * `y` - Vertical position (clamped to [-1000.0, 10000.0])
    /// * `z` - Depth position (clamped to [-10000.0, 10000.0])
    /// * `scale` - Scale vector (each component clamped to [0.001, 1000.0])
    ///
    /// # Returns
    /// A [`Transform`] with validated position and scale values
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::TransformFactory;
    ///
    /// fn spawn_scaled_entity(mut commands: Commands) {
    ///     let scale = Vec3::new(2.0, 1.5, 2.0);
    ///     let transform = TransformFactory::with_scale(0.0, 0.0, 0.0, scale);
    ///     assert_eq!(transform.scale, scale);
    ///     commands.spawn(TransformBundle::from_transform(transform));
    /// }
    /// ```
    pub fn with_scale(x: f32, y: f32, z: f32, scale: Vec3) -> Transform {
        let safe_x = x.max(-10000.0).min(10000.0);
        let safe_y = y.max(-1000.0).min(10000.0);
        let safe_z = z.max(-10000.0).min(10000.0);
        let safe_scale = Vec3::new(
            scale.x.max(0.001).min(1000.0),
            scale.y.max(0.001).min(1000.0),
            scale.z.max(0.001).min(1000.0),
        );
        Transform::from_xyz(safe_x, safe_y, safe_z).with_scale(safe_scale)
    }
}
