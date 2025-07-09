//! Bounding volume implementations for spatial calculations.
//!
//! Provides axis-aligned bounding boxes (AABB) and spheres with efficient
//! intersection tests and spatial operations.
//!
//! # Examples
//!
//! ```rust
//! use amp_math::bounds::{Aabb, Sphere};
//! use glam::Vec3;
//!
//! let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
//! let sphere = Sphere::new(Vec3::ZERO, 1.0);
//!
//! assert!(aabb.intersects_sphere(&sphere));
//! ```

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    /// Minimum corner of the bounding box
    pub min: Vec3,
    /// Maximum corner of the bounding box
    pub max: Vec3,
}

impl Aabb {
    /// Create a new AABB from minimum and maximum points.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// ```
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    /// Create an AABB from a center point and half-extents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let aabb = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::ONE);
    /// ```
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Create an empty AABB (inverted bounds).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    ///
    /// let aabb = Aabb::empty();
    /// assert!(aabb.is_empty());
    /// ```
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Create an AABB that encompasses everything.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    ///
    /// let aabb = Aabb::infinite();
    /// ```
    pub fn infinite() -> Self {
        Self {
            min: Vec3::splat(f32::NEG_INFINITY),
            max: Vec3::splat(f32::INFINITY),
        }
    }

    /// Get the center point of the AABB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// assert_eq!(aabb.center(), Vec3::ZERO);
    /// ```
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size (extents) of the AABB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// assert_eq!(aabb.size(), Vec3::splat(2.0));
    /// ```
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Get the half-extents of the AABB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// assert_eq!(aabb.half_extents(), Vec3::ONE);
    /// ```
    pub fn half_extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// Check if the AABB is empty (has negative volume).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    ///
    /// let empty = Aabb::empty();
    /// assert!(empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Check if a point is inside the AABB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// assert!(aabb.contains_point(Vec3::ZERO));
    /// assert!(!aabb.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    /// ```
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }

    /// Check if this AABB fully contains another AABB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let outer = Aabb::new(Vec3::new(-2.0, -2.0, -2.0), Vec3::new(2.0, 2.0, 2.0));
    /// let inner = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// assert!(outer.contains_aabb(&inner));
    /// ```
    pub fn contains_aabb(&self, other: &Aabb) -> bool {
        other.min.cmpge(self.min).all() && other.max.cmple(self.max).all()
    }

    /// Check if this AABB intersects with another AABB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let aabb1 = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// let aabb2 = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
    /// assert!(aabb1.intersects_aabb(&aabb2));
    /// ```
    pub fn intersects_aabb(&self, other: &Aabb) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }

    /// Check if this AABB intersects with a sphere.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::{Aabb, Sphere};
    /// use glam::Vec3;
    ///
    /// let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// let sphere = Sphere::new(Vec3::ZERO, 1.0);
    /// assert!(aabb.intersects_sphere(&sphere));
    /// ```
    pub fn intersects_sphere(&self, sphere: &Sphere) -> bool {
        let closest_point = sphere.center.clamp(self.min, self.max);
        let distance_squared = (sphere.center - closest_point).length_squared();
        distance_squared <= sphere.radius * sphere.radius
    }

    /// Expand the AABB to include a point.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let mut aabb = Aabb::empty();
    /// aabb.expand_to_include_point(Vec3::new(1.0, 2.0, 3.0));
    /// assert!(aabb.contains_point(Vec3::new(1.0, 2.0, 3.0)));
    /// ```
    pub fn expand_to_include_point(&mut self, point: Vec3) {
        if self.is_empty() {
            self.min = point;
            self.max = point;
        } else {
            self.min = self.min.min(point);
            self.max = self.max.max(point);
        }
    }

    /// Expand the AABB to include another AABB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let mut aabb1 = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// let aabb2 = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
    /// aabb1.expand_to_include_aabb(&aabb2);
    /// assert!(aabb1.contains_aabb(&aabb2));
    /// ```
    pub fn expand_to_include_aabb(&mut self, other: &Aabb) {
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            *self = *other;
        } else {
            self.min = self.min.min(other.min);
            self.max = self.max.max(other.max);
        }
    }

    /// Grow the AABB by a uniform amount in all directions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Aabb;
    /// use glam::Vec3;
    ///
    /// let mut aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    /// aabb.grow(0.5);
    /// assert_eq!(aabb.size(), Vec3::splat(3.0));
    /// ```
    pub fn grow(&mut self, amount: f32) {
        let growth = Vec3::splat(amount);
        self.min -= growth;
        self.max += growth;
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self::empty()
    }
}

/// Sphere in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    /// Center point of the sphere
    pub center: Vec3,
    /// Radius of the sphere
    pub radius: f32,
}

impl Sphere {
    /// Create a new sphere from center and radius.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Sphere;
    /// use glam::Vec3;
    ///
    /// let sphere = Sphere::new(Vec3::ZERO, 1.0);
    /// ```
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self {
            center,
            radius: radius.max(0.0),
        }
    }

    /// Check if a point is inside the sphere.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Sphere;
    /// use glam::Vec3;
    ///
    /// let sphere = Sphere::new(Vec3::ZERO, 1.0);
    /// assert!(sphere.contains_point(Vec3::new(0.5, 0.5, 0.5)));
    /// assert!(!sphere.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    /// ```
    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    /// Check if this sphere fully contains another sphere.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Sphere;
    /// use glam::Vec3;
    ///
    /// let outer = Sphere::new(Vec3::ZERO, 2.0);
    /// let inner = Sphere::new(Vec3::ZERO, 1.0);
    /// assert!(outer.contains_sphere(&inner));
    /// ```
    pub fn contains_sphere(&self, other: &Sphere) -> bool {
        let distance = (self.center - other.center).length();
        distance + other.radius <= self.radius
    }

    /// Check if this sphere intersects with another sphere.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Sphere;
    /// use glam::Vec3;
    ///
    /// let sphere1 = Sphere::new(Vec3::ZERO, 1.0);
    /// let sphere2 = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
    /// assert!(sphere1.intersects_sphere(&sphere2));
    /// ```
    pub fn intersects_sphere(&self, other: &Sphere) -> bool {
        let distance_squared = (self.center - other.center).length_squared();
        let radius_sum = self.radius + other.radius;
        distance_squared <= radius_sum * radius_sum
    }

    /// Get the bounding box of this sphere.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Sphere;
    /// use glam::Vec3;
    ///
    /// let sphere = Sphere::new(Vec3::ZERO, 1.0);
    /// let aabb = sphere.bounding_box();
    /// assert_eq!(aabb.size(), Vec3::splat(2.0));
    /// ```
    pub fn bounding_box(&self) -> Aabb {
        let radius_vec = Vec3::splat(self.radius);
        Aabb::new(self.center - radius_vec, self.center + radius_vec)
    }

    /// Expand the sphere to include a point.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Sphere;
    /// use glam::Vec3;
    ///
    /// let mut sphere = Sphere::new(Vec3::ZERO, 1.0);
    /// sphere.expand_to_include_point(Vec3::new(3.0, 0.0, 0.0));
    /// assert!(sphere.contains_point(Vec3::new(3.0, 0.0, 0.0)));
    /// ```
    pub fn expand_to_include_point(&mut self, point: Vec3) {
        let distance = (point - self.center).length();
        if distance > self.radius {
            self.radius = distance;
        }
    }

    /// Expand the sphere to include another sphere.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::bounds::Sphere;
    /// use glam::Vec3;
    ///
    /// let mut sphere1 = Sphere::new(Vec3::ZERO, 1.0);
    /// let sphere2 = Sphere::new(Vec3::new(3.0, 0.0, 0.0), 1.0);
    /// sphere1.expand_to_include_sphere(&sphere2);
    /// assert!(sphere1.contains_sphere(&sphere2));
    /// ```
    pub fn expand_to_include_sphere(&mut self, other: &Sphere) {
        let distance = (other.center - self.center).length();
        let required_radius = distance + other.radius;
        if required_radius > self.radius {
            self.radius = required_radius;
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = Aabb::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_aabb_swapped_bounds() {
        let aabb = Aabb::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_aabb_from_center_half_extents() {
        let aabb =
            Aabb::from_center_half_extents(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(aabb.min, Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(aabb.max, Vec3::new(1.5, 3.0, 4.5));
    }

    #[test]
    fn test_aabb_empty() {
        let aabb = Aabb::empty();
        assert!(aabb.is_empty());
        assert!(!aabb.contains_point(Vec3::ZERO));
    }

    #[test]
    fn test_aabb_properties() {
        let aabb = Aabb::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.center(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.size(), Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(aabb.half_extents(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_aabb_point_containment() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

        assert!(aabb.contains_point(Vec3::ZERO));
        assert!(aabb.contains_point(Vec3::new(1.0, 1.0, 1.0)));
        assert!(aabb.contains_point(Vec3::new(-1.0, -1.0, -1.0)));
        assert!(!aabb.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_aabb_aabb_containment() {
        let outer = Aabb::new(Vec3::new(-2.0, -2.0, -2.0), Vec3::new(2.0, 2.0, 2.0));
        let inner = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let overlapping = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(3.0, 3.0, 3.0));

        assert!(outer.contains_aabb(&inner));
        assert!(!inner.contains_aabb(&outer));
        assert!(!outer.contains_aabb(&overlapping));
    }

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let aabb2 = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        let aabb3 = Aabb::new(Vec3::new(3.0, 3.0, 3.0), Vec3::new(4.0, 4.0, 4.0));

        assert!(aabb1.intersects_aabb(&aabb2));
        assert!(!aabb1.intersects_aabb(&aabb3));
    }

    #[test]
    fn test_aabb_sphere_intersection() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let sphere_inside = Sphere::new(Vec3::ZERO, 0.5);
        let sphere_intersecting = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let sphere_outside = Sphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0);

        assert!(aabb.intersects_sphere(&sphere_inside));
        assert!(aabb.intersects_sphere(&sphere_intersecting));
        assert!(!aabb.intersects_sphere(&sphere_outside));
    }

    #[test]
    fn test_aabb_expansion() {
        let mut aabb = Aabb::empty();
        aabb.expand_to_include_point(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.min, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));

        aabb.expand_to_include_point(Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_aabb_grow() {
        let mut aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        aabb.grow(0.5);
        assert_eq!(aabb.min, Vec3::new(-1.5, -1.5, -1.5));
        assert_eq!(aabb.max, Vec3::new(1.5, 1.5, 1.5));
    }

    #[test]
    fn test_sphere_creation() {
        let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 5.0);
        assert_eq!(sphere.center, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(sphere.radius, 5.0);

        let negative_radius = Sphere::new(Vec3::ZERO, -1.0);
        assert_eq!(negative_radius.radius, 0.0);
    }

    #[test]
    fn test_sphere_point_containment() {
        let sphere = Sphere::new(Vec3::ZERO, 1.0);

        assert!(sphere.contains_point(Vec3::ZERO));
        assert!(sphere.contains_point(Vec3::new(0.5, 0.5, 0.5)));
        assert!(!sphere.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_sphere_sphere_containment() {
        let outer = Sphere::new(Vec3::ZERO, 2.0);
        let inner = Sphere::new(Vec3::ZERO, 1.0);
        let overlapping = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);

        assert!(outer.contains_sphere(&inner));
        assert!(!inner.contains_sphere(&outer));
        assert!(!outer.contains_sphere(&overlapping));
    }

    #[test]
    fn test_sphere_intersection() {
        let sphere1 = Sphere::new(Vec3::ZERO, 1.0);
        let sphere2 = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let sphere3 = Sphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0);

        assert!(sphere1.intersects_sphere(&sphere2));
        assert!(!sphere1.intersects_sphere(&sphere3));
    }

    #[test]
    fn test_sphere_bounding_box() {
        let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 1.5);
        let aabb = sphere.bounding_box();

        assert_eq!(aabb.min, Vec3::new(-0.5, 0.5, 1.5));
        assert_eq!(aabb.max, Vec3::new(2.5, 3.5, 4.5));
    }

    #[test]
    fn test_sphere_expansion() {
        let mut sphere = Sphere::new(Vec3::ZERO, 1.0);
        sphere.expand_to_include_point(Vec3::new(3.0, 0.0, 0.0));
        assert_eq!(sphere.radius, 3.0);

        let sphere2 = Sphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0);
        sphere.expand_to_include_sphere(&sphere2);
        assert_eq!(sphere.radius, 6.0);
    }
}
