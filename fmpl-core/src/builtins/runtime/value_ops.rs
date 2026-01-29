//! Value operations for code generation.
//!
//! Provides traits that abstract over Value operations for different contexts.
//!
//! - In fmpl-core: Value methods return Result<Value>
//! - In generated standalone code: operations can panic or return Value directly
//!
//! This trait allows the same IR to be transpiled for different targets.

use crate::error::Result;
use crate::value::Value;

/// Trait for value operations that may fail.
/// Used in fmpl-core where operations return Result<Value>.
pub trait FallibleOps {
    fn add(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn sub(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn mul(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn div(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn modulo(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn neg(&self) -> Result<Self>
    where
        Self: Sized;

    fn lt(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn gt(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn le(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    fn ge(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;
}

/// Trait for infallible value operations.
/// Used in standalone generated code where operations panic on error.
pub trait InfallibleOps {
    fn add(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn sub(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn mul(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn div(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn modulo(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn neg(&self) -> Self
    where
        Self: Sized;

    fn lt(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn gt(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn le(&self, other: &Self) -> Self
    where
        Self: Sized;

    fn ge(&self, other: &Self) -> Self
    where
        Self: Sized;
}

// Implement FallibleOps for Value using existing methods
impl FallibleOps for Value {
    fn add(&self, other: &Value) -> Result<Value> {
        Value::add(self, other)
    }

    fn sub(&self, other: &Value) -> Result<Value> {
        Value::sub(self, other)
    }

    fn mul(&self, other: &Value) -> Result<Value> {
        Value::mul(self, other)
    }

    fn div(&self, other: &Value) -> Result<Value> {
        Value::div(self, other)
    }

    fn modulo(&self, other: &Value) -> Result<Value> {
        Value::modulo(self, other)
    }

    fn neg(&self) -> Result<Value> {
        Value::neg(self)
    }

    fn lt(&self, other: &Value) -> Result<Value> {
        Value::lt(self, other)
    }

    fn gt(&self, other: &Value) -> Result<Value> {
        Value::gt(self, other)
    }

    fn le(&self, other: &Value) -> Result<Value> {
        Value::le(self, other)
    }

    fn ge(&self, other: &Value) -> Result<Value> {
        Value::ge(self, other)
    }
}
