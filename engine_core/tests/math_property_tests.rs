use proptest::prelude::*;
use nalgebra as na;
use engine_core::prelude::*;

const EPSILON: f32 = 1e-4;

// Extended math module for property testing
pub mod extended_math {
    pub fn add_vectors(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }
    
    pub fn subtract_vectors(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }
    
    pub fn multiply_scalar(v: [f32; 3], s: f32) -> [f32; 3] {
        [v[0] * s, v[1] * s, v[2] * s]
    }
    
    pub fn dot_product(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }
    
    pub fn cross_product(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0]
        ]
    }
    
    pub fn magnitude(v: [f32; 3]) -> f32 {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }
    
    pub fn normalize(v: [f32; 3]) -> [f32; 3] {
        let mag = magnitude(v);
        if mag > f32::EPSILON {
            [v[0] / mag, v[1] / mag, v[2] / mag]
        } else {
            [0.0, 0.0, 0.0]
        }
    }
    
    pub fn matrix_multiply_3x3(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for (k, b_row) in b.iter().enumerate() {
                    result[i][j] += a[i][k] * b_row[j];
                }
            }
        }
        result
    }
    
    pub fn matrix_transpose_3x3(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
        [
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]]
        ]
    }
    
    pub fn matrix_determinant_3x3(m: [[f32; 3]; 3]) -> f32 {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) -
        m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) +
        m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }
}

use extended_math::*;

fn vector_strategy() -> impl Strategy<Value = [f32; 3]> {
    prop::array::uniform3(-1000.0f32..1000.0f32)
}

fn non_zero_vector_strategy() -> impl Strategy<Value = [f32; 3]> {
    vector_strategy().prop_filter("non-zero", |v| magnitude(*v) > EPSILON)
}

fn scalar_strategy() -> impl Strategy<Value = f32> {
    -1000.0f32..1000.0f32
}

fn matrix_strategy() -> impl Strategy<Value = [[f32; 3]; 3]> {
    prop::array::uniform3(prop::array::uniform3(
        -100.0f32..100.0f32
    ))
}

fn vectors_equal(a: [f32; 3], b: [f32; 3], epsilon: f32) -> bool {
    (a[0] - b[0]).abs() < epsilon &&
    (a[1] - b[1]).abs() < epsilon &&
    (a[2] - b[2]).abs() < epsilon
}

fn matrices_equal(a: [[f32; 3]; 3], b: [[f32; 3]; 3], epsilon: f32) -> bool {
    for i in 0..3 {
        for j in 0..3 {
            if (a[i][j] - b[i][j]).abs() >= epsilon {
                return false;
            }
        }
    }
    true
}

proptest! {
    #[test]
    fn test_vector_addition_commutative(a in vector_strategy(), b in vector_strategy()) {
        let result1 = add_vectors(a, b);
        let result2 = add_vectors(b, a);
        
        // Reference implementation
        let na_a = na::Vector3::new(a[0], a[1], a[2]);
        let na_b = na::Vector3::new(b[0], b[1], b[2]);
        let na_result = na_a + na_b;
        
        prop_assert!(vectors_equal(result1, result2, EPSILON));
        prop_assert!(vectors_equal(result1, [na_result.x, na_result.y, na_result.z], EPSILON));
    }
    
    #[test]
    fn test_vector_addition_associative(a in vector_strategy(), b in vector_strategy(), c in vector_strategy()) {
        let result1 = add_vectors(add_vectors(a, b), c);
        let result2 = add_vectors(a, add_vectors(b, c));
        
        prop_assert!(vectors_equal(result1, result2, EPSILON));
    }
    
    #[test]
    fn test_vector_subtraction_properties(a in vector_strategy(), b in vector_strategy()) {
        let result = subtract_vectors(a, b);
        
        // Reference implementation
        let na_a = na::Vector3::new(a[0], a[1], a[2]);
        let na_b = na::Vector3::new(b[0], b[1], b[2]);
        let na_result = na_a - na_b;
        
        prop_assert!(vectors_equal(result, [na_result.x, na_result.y, na_result.z], EPSILON));
        
        // a - b + b should equal a
        let back_to_a = add_vectors(result, b);
        prop_assert!(vectors_equal(back_to_a, a, EPSILON));
    }
    
    #[test]
    fn test_scalar_multiplication_properties(v in vector_strategy(), s in scalar_strategy()) {
        let result = multiply_scalar(v, s);
        
        // Reference implementation
        let na_v = na::Vector3::new(v[0], v[1], v[2]);
        let na_result = na_v * s;
        
        prop_assert!(vectors_equal(result, [na_result.x, na_result.y, na_result.z], EPSILON));
        
        // Test distributivity: s * (a + b) = s * a + s * b
        let other_v = [1.0, 2.0, 3.0];
        let sum_first = multiply_scalar(add_vectors(v, other_v), s);
        let mult_first = add_vectors(multiply_scalar(v, s), multiply_scalar(other_v, s));
        prop_assert!(vectors_equal(sum_first, mult_first, EPSILON));
    }
    
    #[test]
    fn test_dot_product_properties(a in vector_strategy(), b in vector_strategy()) {
        let result = dot_product(a, b);
        
        // Reference implementation
        let na_a = na::Vector3::new(a[0], a[1], a[2]);
        let na_b = na::Vector3::new(b[0], b[1], b[2]);
        let na_result = na_a.dot(&na_b);
        
        prop_assert!((result - na_result).abs() < EPSILON);
        
        // Commutative property
        let commutative = dot_product(b, a);
        prop_assert!((result - commutative).abs() < EPSILON);
    }
    
    #[test]
    fn test_cross_product_properties(a in vector_strategy(), b in vector_strategy()) {
        let result = cross_product(a, b);
        
        // Reference implementation
        let na_a = na::Vector3::new(a[0], a[1], a[2]);
        let na_b = na::Vector3::new(b[0], b[1], b[2]);
        let na_result = na_a.cross(&na_b);
        
        prop_assert!(vectors_equal(result, [na_result.x, na_result.y, na_result.z], EPSILON));
        
        // Anti-commutative property: a × b = -(b × a)
        let anti_commutative = cross_product(b, a);
        let negated = multiply_scalar(anti_commutative, -1.0);
        prop_assert!(vectors_equal(result, negated, EPSILON));
        
        // Cross product should be orthogonal to both vectors
        if magnitude(result) > EPSILON {
            let dot_a = dot_product(result, a);
            let dot_b = dot_product(result, b);
            prop_assert!(dot_a.abs() < EPSILON);
            prop_assert!(dot_b.abs() < EPSILON);
        }
    }
    
    #[test]
    fn test_vector_normalization(v in non_zero_vector_strategy()) {
        let normalized = normalize(v);
        
        // Reference implementation
        let na_v = na::Vector3::new(v[0], v[1], v[2]);
        let na_normalized = na_v.normalize();
        
        prop_assert!(vectors_equal(normalized, [na_normalized.x, na_normalized.y, na_normalized.z], EPSILON));
        
        // Normalized vector should have magnitude 1
        let mag = magnitude(normalized);
        prop_assert!((mag - 1.0).abs() < EPSILON);
    }
    
    #[test]
    fn test_matrix_multiplication_properties(a in matrix_strategy(), b in matrix_strategy(), c in matrix_strategy()) {
        let result = matrix_multiply_3x3(a, b);
        
        // Reference implementation
        let na_a = na::Matrix3::new(
            a[0][0], a[0][1], a[0][2],
            a[1][0], a[1][1], a[1][2],
            a[2][0], a[2][1], a[2][2]
        );
        let na_b = na::Matrix3::new(
            b[0][0], b[0][1], b[0][2],
            b[1][0], b[1][1], b[1][2],
            b[2][0], b[2][1], b[2][2]
        );
        let na_result = na_a * na_b;
        
        let expected = [
            [na_result[(0, 0)], na_result[(0, 1)], na_result[(0, 2)]],
            [na_result[(1, 0)], na_result[(1, 1)], na_result[(1, 2)]],
            [na_result[(2, 0)], na_result[(2, 1)], na_result[(2, 2)]]
        ];
        
        prop_assert!(matrices_equal(result, expected, EPSILON));
        
        // Test associativity: (AB)C = A(BC)
        let left_assoc = matrix_multiply_3x3(result, c);
        let right_assoc = matrix_multiply_3x3(a, matrix_multiply_3x3(b, c));
        prop_assert!(matrices_equal(left_assoc, right_assoc, EPSILON));
    }
    
    #[test]
    fn test_matrix_transpose_properties(m in matrix_strategy()) {
        let result = matrix_transpose_3x3(m);
        
        // Reference implementation
        let na_m = na::Matrix3::new(
            m[0][0], m[0][1], m[0][2],
            m[1][0], m[1][1], m[1][2],
            m[2][0], m[2][1], m[2][2]
        );
        let na_result = na_m.transpose();
        
        let expected = [
            [na_result[(0, 0)], na_result[(0, 1)], na_result[(0, 2)]],
            [na_result[(1, 0)], na_result[(1, 1)], na_result[(1, 2)]],
            [na_result[(2, 0)], na_result[(2, 1)], na_result[(2, 2)]]
        ];
        
        prop_assert!(matrices_equal(result, expected, EPSILON));
        
        // Double transpose should return original
        let double_transpose = matrix_transpose_3x3(result);
        prop_assert!(matrices_equal(double_transpose, m, EPSILON));
    }
    
    #[test]
    fn test_matrix_determinant_properties(m in matrix_strategy()) {
        let result = matrix_determinant_3x3(m);
        
        // Reference implementation
        let na_m = na::Matrix3::new(
            m[0][0], m[0][1], m[0][2],
            m[1][0], m[1][1], m[1][2],
            m[2][0], m[2][1], m[2][2]
        );
        let na_result = na_m.determinant();
        
        prop_assert!((result - na_result).abs() < EPSILON);
        
        // Determinant of transpose should equal determinant of original
        let transposed = matrix_transpose_3x3(m);
        let transpose_det = matrix_determinant_3x3(transposed);
        prop_assert!((result - transpose_det).abs() < EPSILON);
    }
    
    #[test]
    fn test_clamp_f32_properties(value in any::<f32>(), min in any::<f32>(), max in any::<f32>()) {
        prop_assume!(min <= max);
        prop_assume!(min.is_finite() && max.is_finite());
        
        let result = clamp_f32(value, min, max);
        
        // Result should be within bounds
        prop_assert!(result >= min);
        prop_assert!(result <= max);
        
        // If value is within bounds, result should equal value
        if value.is_finite() && value >= min && value <= max {
            prop_assert_eq!(result, value);
        }
        
        // If value is below min, result should be min
        if value.is_finite() && value < min {
            prop_assert_eq!(result, min);
        }
        
        // If value is above max, result should be max
        if value.is_finite() && value > max {
            prop_assert_eq!(result, max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_math_operations() {
        // Basic sanity checks
        assert_eq!(add_vectors([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), [5.0, 7.0, 9.0]);
        assert_eq!(subtract_vectors([4.0, 5.0, 6.0], [1.0, 2.0, 3.0]), [3.0, 3.0, 3.0]);
        assert_eq!(multiply_scalar([1.0, 2.0, 3.0], 2.0), [2.0, 4.0, 6.0]);
        assert_eq!(dot_product([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
        
        let cross = cross_product([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(vectors_equal(cross, [0.0, 0.0, 1.0], EPSILON));
        
        let mag = magnitude([3.0, 4.0, 0.0]);
        assert!((mag - 5.0).abs() < EPSILON);
        
        assert_eq!(clamp_f32(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp_f32(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp_f32(15.0, 0.0, 10.0), 10.0);
    }
}
