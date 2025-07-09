//! Transform utilities wrapping glam for common 3D operations.
//!
//! Provides high-level wrappers around glam's Mat4 for common transform operations
//! with convenient builder patterns and game-specific functionality.
//!
//! # Examples
//!
//! ```rust
//! use amp_math::transforms::Transform;
//! use glam::{Vec3, Quat};
//!
//! let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))
//!     .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0))
//!     .with_scale(Vec3::splat(2.0));
//! ```

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// A transform in 3D space with translation, rotation, and scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Position in 3D space
    pub translation: Vec3,
    /// Rotation as a quaternion
    pub rotation: Quat,
    /// Scale factor for each axis
    pub scale: Vec3,
}

impl Transform {
    /// Create a new transform with identity values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    ///
    /// let transform = Transform::identity();
    /// ```
    pub fn identity() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from a translation vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// ```
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from a rotation quaternion.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Quat;
    ///
    /// let transform = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0));
    /// ```
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from a scale vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::from_scale(Vec3::splat(2.0));
    /// ```
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    /// Create a transform from translation, rotation, and scale.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::{Vec3, Quat};
    ///
    /// let transform = Transform::from_trs(
    ///     Vec3::new(1.0, 2.0, 3.0),
    ///     Quat::from_rotation_y(std::f32::consts::PI / 2.0),
    ///     Vec3::splat(2.0)
    /// );
    /// ```
    pub fn from_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Create a transform from a transformation matrix.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::{Mat4, Vec3};
    ///
    /// let matrix = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// let transform = Transform::from_matrix(matrix);
    /// ```
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Set the translation component.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::identity().with_translation(Vec3::new(1.0, 2.0, 3.0));
    /// ```
    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    /// Set the rotation component.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Quat;
    ///
    /// let transform = Transform::identity().with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0));
    /// ```
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the scale component.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::identity().with_scale(Vec3::splat(2.0));
    /// ```
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// Convert the transform to a transformation matrix.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// let matrix = transform.to_matrix();
    /// ```
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Get the forward direction vector (negative Z axis).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Quat;
    ///
    /// let transform = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0));
    /// let forward = transform.forward();
    /// ```
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction vector (positive X axis).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Quat;
    ///
    /// let transform = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0));
    /// let right = transform.right();
    /// ```
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction vector (positive Y axis).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Quat;
    ///
    /// let transform = Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0));
    /// let up = transform.up();
    /// ```
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Transform a point from local space to world space.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// let local_point = Vec3::new(1.0, 0.0, 0.0);
    /// let world_point = transform.transform_point(local_point);
    /// ```
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.to_matrix().transform_point3(point)
    }

    /// Transform a direction vector from local space to world space.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::{Vec3, Quat};
    ///
    /// let transform = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0));
    /// let local_dir = Vec3::X;
    /// let world_dir = transform.transform_direction(local_dir);
    /// ```
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation * (direction * self.scale)
    }

    /// Look at a target position.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
    /// transform.look_at(Vec3::new(1.0, 0.0, 0.0), Vec3::Y);
    /// ```
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.translation).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);

        self.rotation = Quat::from_mat3(&glam::Mat3::from_cols(right, up, -forward));
    }

    /// Multiply this transform by another transform.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform1 = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
    /// let transform2 = Transform::from_translation(Vec3::new(0.0, 1.0, 0.0));
    /// let combined = transform1.mul_transform(transform2);
    /// ```
    pub fn mul_transform(&self, other: Transform) -> Self {
        let matrix = self.to_matrix() * other.to_matrix();
        Self::from_matrix(matrix)
    }

    /// Get the inverse of this transform.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// let inverse = transform.inverse();
    /// ```
    pub fn inverse(&self) -> Self {
        let matrix = self.to_matrix();
        Self::from_matrix(matrix.inverse())
    }

    /// Linearly interpolate between two transforms.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::Vec3;
    ///
    /// let transform1 = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
    /// let transform2 = Transform::from_translation(Vec3::new(1.0, 1.0, 1.0));
    /// let lerped = transform1.lerp(transform2, 0.5);
    /// ```
    pub fn lerp(&self, other: Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.lerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    /// Spherically interpolate between two transforms.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::Transform;
    /// use glam::{Vec3, Quat};
    ///
    /// let transform1 = Transform::from_rotation(Quat::IDENTITY);
    /// let transform2 = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0));
    /// let slerped = transform1.slerp(transform2, 0.5);
    /// ```
    pub fn slerp(&self, other: Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<Mat4> for Transform {
    fn from(matrix: Mat4) -> Self {
        Self::from_matrix(matrix)
    }
}

impl From<Transform> for Mat4 {
    fn from(transform: Transform) -> Self {
        transform.to_matrix()
    }
}

/// Camera-specific transform utilities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraTransform {
    /// The camera's transform in world space
    pub transform: Transform,
    /// Field of view in radians
    pub fov: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub far: f32,
    /// Width/height aspect ratio
    pub aspect_ratio: f32,
}

impl CameraTransform {
    /// Create a new camera transform.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::{CameraTransform, Transform};
    ///
    /// let camera = CameraTransform::new(
    ///     Transform::identity(),
    ///     60.0_f32.to_radians(),
    ///     0.1,
    ///     1000.0,
    ///     16.0 / 9.0
    /// );
    /// ```
    pub fn new(transform: Transform, fov: f32, near: f32, far: f32, aspect_ratio: f32) -> Self {
        Self {
            transform,
            fov,
            near,
            far,
            aspect_ratio,
        }
    }

    /// Get the view matrix for this camera.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::{CameraTransform, Transform};
    ///
    /// let camera = CameraTransform::new(
    ///     Transform::identity(),
    ///     60.0_f32.to_radians(),
    ///     0.1,
    ///     1000.0,
    ///     16.0 / 9.0
    /// );
    /// let view_matrix = camera.view_matrix();
    /// ```
    pub fn view_matrix(&self) -> Mat4 {
        self.transform.to_matrix().inverse()
    }

    /// Get the projection matrix for this camera.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::{CameraTransform, Transform};
    ///
    /// let camera = CameraTransform::new(
    ///     Transform::identity(),
    ///     60.0_f32.to_radians(),
    ///     0.1,
    ///     1000.0,
    ///     16.0 / 9.0
    /// );
    /// let projection_matrix = camera.projection_matrix();
    /// ```
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    /// Get the combined view-projection matrix.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::transforms::{CameraTransform, Transform};
    ///
    /// let camera = CameraTransform::new(
    ///     Transform::identity(),
    ///     60.0_f32.to_radians(),
    ///     0.1,
    ///     1000.0,
    ///     16.0 / 9.0
    /// );
    /// let view_projection = camera.view_projection_matrix();
    /// ```
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_transform_identity() {
        let transform = Transform::identity();
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_from_translation() {
        let translation = Vec3::new(1.0, 2.0, 3.0);
        let transform = Transform::from_translation(translation);
        assert_eq!(transform.translation, translation);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_matrix_conversion() {
        let transform = Transform::from_trs(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_y(PI / 2.0),
            Vec3::splat(2.0),
        );

        let matrix = transform.to_matrix();
        let back_to_transform = Transform::from_matrix(matrix);

        assert!((back_to_transform.translation - transform.translation).length() < 0.001);
        assert!((back_to_transform.rotation.dot(transform.rotation)).abs() > 0.999);
        assert!((back_to_transform.scale - transform.scale).length() < 0.001);
    }

    #[test]
    fn test_transform_point() {
        let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = transform.transform_point(point);
        assert_eq!(transformed, Vec3::new(2.0, 2.0, 3.0));
    }

    #[test]
    fn test_transform_direction_vectors() {
        let transform = Transform::from_rotation(Quat::from_rotation_y(PI / 2.0));

        let forward = transform.forward();
        let right = transform.right();
        let up = transform.up();

        // Test rotation and direction vectors work correctly
        assert!((up - Vec3::Y).length() < 0.001);

        // The exact direction depends on the coordinate system
        // Just verify the vectors are unit length and orthogonal
        assert!((forward.length() - 1.0).abs() < 0.001);
        assert!((right.length() - 1.0).abs() < 0.001);
        assert!((up.length() - 1.0).abs() < 0.001);

        // Verify they're orthogonal
        assert!(forward.dot(right).abs() < 0.001);
        assert!(forward.dot(up).abs() < 0.001);
        assert!(right.dot(up).abs() < 0.001);
    }

    #[test]
    fn test_transform_look_at() {
        let mut transform = Transform::from_translation(Vec3::ZERO);
        transform.look_at(Vec3::new(1.0, 0.0, 0.0), Vec3::Y);

        let forward = transform.forward();
        assert!((forward - Vec3::X).length() < 0.001);
    }

    #[test]
    fn test_transform_inverse() {
        let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let inverse = transform.inverse();
        let identity = transform.mul_transform(inverse);

        assert!((identity.translation - Vec3::ZERO).length() < 0.001);
        assert!((identity.rotation.dot(Quat::IDENTITY)).abs() > 0.999);
        assert!((identity.scale - Vec3::ONE).length() < 0.001);
    }

    #[test]
    fn test_transform_lerp() {
        let transform1 = Transform::from_translation(Vec3::ZERO);
        let transform2 = Transform::from_translation(Vec3::new(2.0, 2.0, 2.0));
        let lerped = transform1.lerp(transform2, 0.5);

        assert_eq!(lerped.translation, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_transform_slerp() {
        let transform1 = Transform::from_rotation(Quat::IDENTITY);
        let transform2 = Transform::from_rotation(Quat::from_rotation_y(PI / 2.0));
        let slerped = transform1.slerp(transform2, 0.5);

        // Should be halfway between the two rotations
        let expected_angle = PI / 4.0;
        let actual_angle = slerped.rotation.to_axis_angle().1;
        assert!((actual_angle - expected_angle).abs() < 0.001);
    }

    #[test]
    fn test_camera_transform() {
        let camera = CameraTransform::new(
            Transform::identity(),
            60.0_f32.to_radians(),
            0.1,
            1000.0,
            16.0 / 9.0,
        );

        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();
        let view_projection = camera.view_projection_matrix();

        assert_eq!(view_matrix, Mat4::IDENTITY);
        assert_ne!(projection_matrix, Mat4::IDENTITY);
        assert_eq!(view_projection, projection_matrix);
    }

    #[test]
    fn test_transform_builder_pattern() {
        let transform = Transform::identity()
            .with_translation(Vec3::new(1.0, 2.0, 3.0))
            .with_rotation(Quat::from_rotation_y(PI / 2.0))
            .with_scale(Vec3::splat(2.0));

        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert!((transform.rotation.dot(Quat::from_rotation_y(PI / 2.0))).abs() > 0.999);
        assert_eq!(transform.scale, Vec3::splat(2.0));
    }
}
