//! Random number generation built-ins for FMPL.

use crate::error::Result;
use crate::value::Value;
use rand::Rng;

/// The rand built-in object for random number generation.
pub struct RandBuiltin;

impl RandBuiltin {
    /// Generate a random integer in the range [min, max].
    ///
    /// Arguments:
    /// - min: Minimum value (inclusive, integer)
    /// - max: Maximum value (inclusive, integer)
    ///
    /// Returns a random integer.
    ///
    /// # Notes
    ///
    /// - If min >= max, the bounds are automatically swapped
    /// - Uses the rand crate's thread_rng() for randomness
    /// - The range is [min, max) - max is exclusive
    pub fn int(min: i64, max: i64) -> Result<Value> {
        let mut rng = rand::thread_rng();
        let random = rng.gen_range(min..max);
        Ok(Value::Int(random))
    }

    /// Generate a random float in the range [0.0, 1.0).
    ///
    /// Returns a random float between 0.0 (inclusive) and 1.0 (exclusive).
    ///
    /// # Notes
    ///
    /// - Uses the rand crate's thread_rng() for randomness
    pub fn float() -> Result<Value> {
        let mut rng = rand::thread_rng();
        let random: f64 = rng.gen_range(0.0..1.0);
        Ok(Value::Float(random))
    }
}
