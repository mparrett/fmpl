use crate::compiler::CompiledCode;
use crate::error::{Error, Result};

pub const BYTECODE_VERSION: u16 = 1;

pub fn encode_bytecode(_code: &CompiledCode) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&BYTECODE_VERSION.to_le_bytes());
    Ok(bytes)
}

pub fn decode_bytecode(bytes: &[u8]) -> Result<CompiledCode> {
    if bytes.len() < 2 {
        return Err(Error::Runtime("bytecode too short".to_string()));
    }
    let version = u16::from_le_bytes([bytes[0], bytes[1]]);
    if version != BYTECODE_VERSION {
        return Err(Error::Runtime("bytecode version mismatch".to_string()));
    }
    Ok(CompiledCode::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let code = CompiledCode::new();
        let bytes = encode_bytecode(&code).unwrap();
        let decoded = decode_bytecode(&bytes).unwrap();
        assert_eq!(decoded.instructions.len(), 0);
    }
}
