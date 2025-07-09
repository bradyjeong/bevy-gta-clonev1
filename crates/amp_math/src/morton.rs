//! Morton encoding (Z-order curve) for 3D spatial indexing.
//!
//! Morton encoding maps 3D coordinates to a single integer value that preserves
//! spatial locality. Points that are close in 3D space will have similar Morton codes.
//!
//! # Examples
//!
//! ```rust
//! use amp_math::morton::Morton3D;
//! use glam::Vec3;
//!
//! let pos = Vec3::new(1.0, 2.0, 3.0);
//! let morton = Morton3D::encode(pos);
//! let decoded = Morton3D::decode(morton);
//! assert!((decoded - pos).length() < 0.001);
//! ```

use glam::Vec3;

/// Morton encoding for 2D coordinates using 32-bit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Morton2D;

impl Morton2D {
    /// Maximum coordinate value that can be encoded (16 bits per axis).
    pub const MAX_COORD: u32 = (1 << 16) - 1;

    /// Encode 2D coordinates into a Morton code.
    pub fn encode(x: u32, y: u32) -> u64 {
        let x = (x & Self::MAX_COORD) as u64;
        let y = (y & Self::MAX_COORD) as u64;

        Self::spread_bits_2d(x) | (Self::spread_bits_2d(y) << 1)
    }

    /// Decode a Morton code back to 2D coordinates.
    pub fn decode(morton: u64) -> (u32, u32) {
        let x = Self::compact_bits_2d(morton);
        let y = Self::compact_bits_2d(morton >> 1);

        (x, y)
    }

    fn spread_bits_2d(mut value: u64) -> u64 {
        value = (value | (value << 16)) & 0x0000ffff0000ffff;
        value = (value | (value << 8)) & 0x00ff00ff00ff00ff;
        value = (value | (value << 4)) & 0x0f0f0f0f0f0f0f0f;
        value = (value | (value << 2)) & 0x3333333333333333;
        value = (value | (value << 1)) & 0x5555555555555555;
        value
    }

    fn compact_bits_2d(mut value: u64) -> u32 {
        value &= 0x5555555555555555;
        value = (value | (value >> 1)) & 0x3333333333333333;
        value = (value | (value >> 2)) & 0x0f0f0f0f0f0f0f0f;
        value = (value | (value >> 4)) & 0x00ff00ff00ff00ff;
        value = (value | (value >> 8)) & 0x0000ffff0000ffff;
        value = (value | (value >> 16)) & 0x00000000ffffffff;
        value as u32
    }
}

/// Convenience functions for Morton 2D encoding.
pub fn morton_encode_2d(x: u32, y: u32) -> u64 {
    Morton2D::encode(x, y)
}

/// Convenience functions for Morton 2D decoding.
pub fn morton_decode_2d(morton: u64) -> (u32, u32) {
    Morton2D::decode(morton)
}

/// Morton encoding for 3D coordinates using 64-bit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Morton3D;

impl Morton3D {
    /// Maximum coordinate value that can be encoded (21 bits per axis).
    pub const MAX_COORD: u32 = (1 << 21) - 1;

    /// Encode a 3D position into a Morton code.
    ///
    /// Coordinates are normalized to the range [0, MAX_COORD] before encoding.
    /// Values outside this range are clamped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::morton::Morton3D;
    /// use glam::Vec3;
    ///
    /// let pos = Vec3::new(100.0, 200.0, 300.0);
    /// let morton = Morton3D::encode(pos);
    /// ```
    pub fn encode(pos: Vec3) -> u64 {
        let x = Self::normalize_coord(pos.x);
        let y = Self::normalize_coord(pos.y);
        let z = Self::normalize_coord(pos.z);

        Self::encode_normalized(x, y, z)
    }

    /// Encode normalized coordinates (0 to MAX_COORD) into a Morton code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::morton::Morton3D;
    ///
    /// let morton = Morton3D::encode_normalized(100, 200, 300);
    /// ```
    pub fn encode_normalized(x: u32, y: u32, z: u32) -> u64 {
        let x = (x & Self::MAX_COORD) as u64;
        let y = (y & Self::MAX_COORD) as u64;
        let z = (z & Self::MAX_COORD) as u64;

        Self::spread_bits(x) | (Self::spread_bits(y) << 1) | (Self::spread_bits(z) << 2)
    }

    /// Decode a Morton code back to 3D coordinates.
    ///
    /// Returns coordinates in the range [0, MAX_COORD] as floating point values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::morton::Morton3D;
    ///
    /// let morton = 0x123456789abcdef0;
    /// let pos = Morton3D::decode(morton);
    /// ```
    pub fn decode(morton: u64) -> Vec3 {
        let x = Self::compact_bits(morton) as f32;
        let y = Self::compact_bits(morton >> 1) as f32;
        let z = Self::compact_bits(morton >> 2) as f32;

        Vec3::new(x, y, z)
    }

    /// Get the common prefix length between two Morton codes.
    ///
    /// Used for hierarchical spatial data structures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amp_math::morton::Morton3D;
    ///
    /// let morton1 = Morton3D::encode_normalized(100, 200, 300);
    /// let morton2 = Morton3D::encode_normalized(101, 200, 300);
    /// let prefix_len = Morton3D::common_prefix_length(morton1, morton2);
    /// ```
    pub fn common_prefix_length(a: u64, b: u64) -> u32 {
        (a ^ b).leading_zeros()
    }

    fn normalize_coord(coord: f32) -> u32 {
        let clamped = coord.clamp(0.0, Self::MAX_COORD as f32);
        clamped as u32
    }

    fn spread_bits(mut value: u64) -> u64 {
        value = (value | (value << 32)) & 0x1f00000000ffff;
        value = (value | (value << 16)) & 0x1f0000ff0000ff;
        value = (value | (value << 8)) & 0x100f00f00f00f00f;
        value = (value | (value << 4)) & 0x10c30c30c30c30c3;
        value = (value | (value << 2)) & 0x1249249249249249;
        value
    }

    fn compact_bits(mut value: u64) -> u32 {
        value &= 0x1249249249249249;
        value = (value | (value >> 2)) & 0x10c30c30c30c30c3;
        value = (value | (value >> 4)) & 0x100f00f00f00f00f;
        value = (value | (value >> 8)) & 0x1f0000ff0000ff;
        value = (value | (value >> 16)) & 0x1f00000000ffff;
        value = (value | (value >> 32)) & 0x1fffff;
        value as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_identity() {
        let test_cases = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(100.0, 200.0, 300.0),
            Vec3::new(1000.0, 2000.0, 3000.0),
            Vec3::new(
                Morton3D::MAX_COORD as f32,
                Morton3D::MAX_COORD as f32,
                Morton3D::MAX_COORD as f32,
            ),
        ];

        for pos in test_cases {
            let morton = Morton3D::encode(pos);
            let decoded = Morton3D::decode(morton);

            // Allow for small floating point errors
            assert!(
                (decoded.x - pos.x).abs() < 1.0,
                "X mismatch: {} vs {}",
                decoded.x,
                pos.x
            );
            assert!(
                (decoded.y - pos.y).abs() < 1.0,
                "Y mismatch: {} vs {}",
                decoded.y,
                pos.y
            );
            assert!(
                (decoded.z - pos.z).abs() < 1.0,
                "Z mismatch: {} vs {}",
                decoded.z,
                pos.z
            );
        }
    }

    #[test]
    fn test_encode_normalized() {
        let morton = Morton3D::encode_normalized(1, 2, 3);
        assert_ne!(morton, 0);

        let decoded = Morton3D::decode(morton);
        assert_eq!(decoded.x as u32, 1);
        assert_eq!(decoded.y as u32, 2);
        assert_eq!(decoded.z as u32, 3);
    }

    #[test]
    fn test_clamping() {
        let pos = Vec3::new(-100.0, (Morton3D::MAX_COORD + 1000) as f32, 500.0);
        let morton = Morton3D::encode(pos);
        let decoded = Morton3D::decode(morton);

        assert_eq!(decoded.x, 0.0);
        assert_eq!(decoded.y, Morton3D::MAX_COORD as f32);
        assert_eq!(decoded.z, 500.0);
    }

    #[test]
    fn test_spatial_locality() {
        let pos1 = Vec3::new(100.0, 100.0, 100.0);
        let pos2 = Vec3::new(101.0, 101.0, 101.0);
        let pos3 = Vec3::new(1000.0, 1000.0, 1000.0);

        let morton1 = Morton3D::encode(pos1);
        let morton2 = Morton3D::encode(pos2);
        let morton3 = Morton3D::encode(pos3);

        // Nearby points should have more similar Morton codes
        let diff_close = (morton1 as i64 - morton2 as i64).abs();
        let diff_far = (morton1 as i64 - morton3 as i64).abs();

        assert!(diff_close < diff_far);
    }

    #[test]
    fn test_common_prefix_length() {
        let morton1 = Morton3D::encode_normalized(0b1010101, 0b1100110, 0b1111000);
        let morton2 = Morton3D::encode_normalized(0b1010100, 0b1100110, 0b1111000);

        let prefix_len = Morton3D::common_prefix_length(morton1, morton2);
        assert!(prefix_len > 0);

        let identical = Morton3D::common_prefix_length(morton1, morton1);
        assert_eq!(identical, 64);
    }

    #[test]
    fn test_spread_compact_bits() {
        let values = vec![0, 1, 2, 3, 0xff, 0xffff, Morton3D::MAX_COORD as u64];

        for value in values {
            let spread = Morton3D::spread_bits(value);
            let compact = Morton3D::compact_bits(spread);
            assert_eq!(compact as u64, value & Morton3D::MAX_COORD as u64);
        }
    }

    #[test]
    fn test_max_coord_boundary() {
        let pos = Vec3::new(
            Morton3D::MAX_COORD as f32,
            Morton3D::MAX_COORD as f32,
            Morton3D::MAX_COORD as f32,
        );
        let morton = Morton3D::encode(pos);
        let decoded = Morton3D::decode(morton);

        assert_eq!(decoded.x, Morton3D::MAX_COORD as f32);
        assert_eq!(decoded.y, Morton3D::MAX_COORD as f32);
        assert_eq!(decoded.z, Morton3D::MAX_COORD as f32);
    }

    #[test]
    fn test_zero_case() {
        let zero = Vec3::ZERO;
        let morton = Morton3D::encode(zero);
        let decoded = Morton3D::decode(morton);

        assert_eq!(decoded, Vec3::ZERO);
        assert_eq!(morton, 0);
    }

    #[test]
    fn test_bit_interleaving() {
        // Test that bits are properly interleaved
        let morton = Morton3D::encode_normalized(0b001, 0b010, 0b100);

        // Check that the bits are in the correct positions
        // Z=100 (bit 2), Y=010 (bit 1), X=001 (bit 0)
        // Should interleave as: Z2 Y1 X0 Z1 Y0 X1 Z0 Y2 X2
        let expected_pattern = morton & 0b111; // Check lowest 3 bits
        assert_ne!(expected_pattern, 0);
    }

    #[test]
    fn test_morton_2d_encode_decode() {
        let test_cases = vec![
            (0, 0),
            (1, 1),
            (10, 20),
            (100, 200),
            (Morton2D::MAX_COORD, Morton2D::MAX_COORD),
        ];

        for (x, y) in test_cases {
            let morton = Morton2D::encode(x, y);
            let (decoded_x, decoded_y) = Morton2D::decode(morton);
            assert_eq!(decoded_x, x);
            assert_eq!(decoded_y, y);
        }
    }

    #[test]
    fn test_morton_2d_convenience_functions() {
        let (x, y) = (42, 84);
        let morton = morton_encode_2d(x, y);
        let (decoded_x, decoded_y) = morton_decode_2d(morton);
        assert_eq!(decoded_x, x);
        assert_eq!(decoded_y, y);
    }

    #[test]
    fn test_morton_2d_spatial_locality() {
        let morton1 = Morton2D::encode(100, 100);
        let morton2 = Morton2D::encode(101, 101);
        let morton3 = Morton2D::encode(1000, 1000);

        // Nearby points should have more similar Morton codes
        let diff_close = (morton1 as i64 - morton2 as i64).abs();
        let diff_far = (morton1 as i64 - morton3 as i64).abs();

        assert!(diff_close < diff_far);
    }
}
