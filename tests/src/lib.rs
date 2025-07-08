//! Integration tests for the GTA game
#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

/// Simple addition function for testing
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
