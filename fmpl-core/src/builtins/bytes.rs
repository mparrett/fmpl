//! Bytes encoding builtin for FMPL.
//!
//! Provides primitives for encoding integers to byte sequences,
//! including LEB128 variable-length encoding used by execution_tape bytecode.

use crate::value::Value;
use std::sync::Arc;

/// Encode an unsigned integer as ULEB128.
/// Returns a list of bytes.
pub fn uleb128(n: i64) -> Value {
    let mut bytes = Vec::new();
    let mut val = n as u64;
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80; // More bytes to come
        }
        bytes.push(Value::Int(byte as i64));
        if val == 0 {
            break;
        }
    }
    Value::List(Arc::new(bytes))
}

/// Encode a signed integer as SLEB128.
/// Returns a list of bytes.
pub fn sleb128(n: i64) -> Value {
    let mut bytes = Vec::new();
    let mut val = n;
    let mut more = true;
    while more {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        // Sign bit of byte is second high order bit (0x40)
        let sign_bit = (byte & 0x40) != 0;
        if (val == 0 && !sign_bit) || (val == -1 && sign_bit) {
            more = false;
        } else {
            byte |= 0x80; // More bytes to come
        }
        bytes.push(Value::Int(byte as i64));
    }
    Value::List(Arc::new(bytes))
}

/// Encode as single byte.
pub fn u8_encode(n: i64) -> Value {
    Value::List(Arc::new(vec![Value::Int(n & 0xFF)]))
}

/// Encode as 16-bit little-endian.
pub fn u16_le(n: i64) -> Value {
    let val = n as u16;
    Value::List(Arc::new(vec![
        Value::Int((val & 0xFF) as i64),
        Value::Int(((val >> 8) & 0xFF) as i64),
    ]))
}

/// Encode as 32-bit little-endian.
pub fn u32_le(n: i64) -> Value {
    let val = n as u32;
    Value::List(Arc::new(vec![
        Value::Int((val & 0xFF) as i64),
        Value::Int(((val >> 8) & 0xFF) as i64),
        Value::Int(((val >> 16) & 0xFF) as i64),
        Value::Int(((val >> 24) & 0xFF) as i64),
    ]))
}

/// Encode as 64-bit little-endian.
pub fn u64_le(n: i64) -> Value {
    let val = n as u64;
    Value::List(Arc::new(vec![
        Value::Int((val & 0xFF) as i64),
        Value::Int(((val >> 8) & 0xFF) as i64),
        Value::Int(((val >> 16) & 0xFF) as i64),
        Value::Int(((val >> 24) & 0xFF) as i64),
        Value::Int(((val >> 32) & 0xFF) as i64),
        Value::Int(((val >> 40) & 0xFF) as i64),
        Value::Int(((val >> 48) & 0xFF) as i64),
        Value::Int(((val >> 56) & 0xFF) as i64),
    ]))
}

/// Encode f64 bits as 64-bit little-endian.
pub fn f64_le(bits: u64) -> Value {
    Value::List(Arc::new(vec![
        Value::Int((bits & 0xFF) as i64),
        Value::Int(((bits >> 8) & 0xFF) as i64),
        Value::Int(((bits >> 16) & 0xFF) as i64),
        Value::Int(((bits >> 24) & 0xFF) as i64),
        Value::Int(((bits >> 32) & 0xFF) as i64),
        Value::Int(((bits >> 40) & 0xFF) as i64),
        Value::Int(((bits >> 48) & 0xFF) as i64),
        Value::Int(((bits >> 56) & 0xFF) as i64),
    ]))
}

/// Concatenate a list of byte lists into a single byte list.
pub fn concat(lists: &Value) -> Value {
    let mut result = Vec::new();
    if let Value::List(items) = lists {
        for item in items.iter() {
            if let Value::List(bytes) = item {
                result.extend(bytes.iter().cloned());
            } else if let Value::Int(b) = item {
                // Allow single bytes in the list too
                result.push(Value::Int(*b));
            }
        }
    }
    Value::List(Arc::new(result))
}

/// Convert a list of byte values to a String (hex representation for debugging).
pub fn to_hex(list: &Value) -> Value {
    if let Value::List(items) = list {
        let hex: String = items
            .iter()
            .filter_map(|v| {
                if let Value::Int(n) = v {
                    Some(format!("{:02x}", *n as u8))
                } else {
                    None
                }
            })
            .collect();
        Value::String(smol_str::SmolStr::new(hex))
    } else {
        Value::String(smol_str::SmolStr::new(""))
    }
}

/// Get the length of a byte list.
pub fn len(list: &Value) -> Value {
    if let Value::List(items) = list {
        Value::Int(items.len() as i64)
    } else {
        Value::Int(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes_to_vec(v: &Value) -> Vec<u8> {
        if let Value::List(items) = v {
            items
                .iter()
                .filter_map(|v| {
                    if let Value::Int(n) = v {
                        Some(*n as u8)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    #[test]
    fn test_uleb128_small() {
        assert_eq!(bytes_to_vec(&uleb128(0)), vec![0x00]);
        assert_eq!(bytes_to_vec(&uleb128(1)), vec![0x01]);
        assert_eq!(bytes_to_vec(&uleb128(127)), vec![0x7F]);
    }

    #[test]
    fn test_uleb128_larger() {
        assert_eq!(bytes_to_vec(&uleb128(128)), vec![0x80, 0x01]);
        assert_eq!(bytes_to_vec(&uleb128(255)), vec![0xFF, 0x01]);
        assert_eq!(bytes_to_vec(&uleb128(256)), vec![0x80, 0x02]);
        assert_eq!(bytes_to_vec(&uleb128(16383)), vec![0xFF, 0x7F]);
        assert_eq!(bytes_to_vec(&uleb128(16384)), vec![0x80, 0x80, 0x01]);
    }

    #[test]
    fn test_sleb128_positive() {
        assert_eq!(bytes_to_vec(&sleb128(0)), vec![0x00]);
        assert_eq!(bytes_to_vec(&sleb128(1)), vec![0x01]);
        assert_eq!(bytes_to_vec(&sleb128(63)), vec![0x3F]);
        assert_eq!(bytes_to_vec(&sleb128(64)), vec![0xC0, 0x00]);
    }

    #[test]
    fn test_sleb128_negative() {
        assert_eq!(bytes_to_vec(&sleb128(-1)), vec![0x7F]);
        assert_eq!(bytes_to_vec(&sleb128(-64)), vec![0x40]);
        assert_eq!(bytes_to_vec(&sleb128(-65)), vec![0xBF, 0x7F]);
        assert_eq!(bytes_to_vec(&sleb128(-128)), vec![0x80, 0x7F]);
    }

    #[test]
    fn test_u16_le() {
        assert_eq!(bytes_to_vec(&u16_le(0x1234)), vec![0x34, 0x12]);
    }

    #[test]
    fn test_u32_le() {
        assert_eq!(
            bytes_to_vec(&u32_le(0x12345678)),
            vec![0x78, 0x56, 0x34, 0x12]
        );
    }
}
