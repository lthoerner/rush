use snafu::Snafu;
use wasmtime::Val;

pub mod manager;

/// The address of a byte that may be controlled by the WASM engine.
pub type WasmPtr = u32;

#[derive(Debug, Snafu)]
/// Val must be an i64 to convert to a WasmSpan
pub struct TryFromWasmError;

/// A span of bytes that may be controlled by the WASM engine.
/// This can be passed to a [`WasmMemoryManager`]
/// to be read or written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmSpan {
    pub offset: WasmPtr,
    pub length: WasmPtr,
}

impl WasmSpan {
    /// Returns an i64 where the first half is the offset of the memory and the 2nd is its length
    pub fn to_wasm(&self) -> Val {
        // For the plugin to use our data, it needs to know where to find
        // it and how long it is. In this implementation, we represent buffers as
        // a 64 bit number where the first half is the pointer to the buffer
        // and the second half is its length.
        let ptr = (self.offset as u64) << 32 | (self.length as u64);
        Val::I64(ptr as i64)
    }

    /// Parses an i64 where the first half is the offset and the 2nd is the length
    pub fn try_from_wasm(ptr: &Val) -> Result<Self, TryFromWasmError> {
        let ptr = ptr.i64().ok_or(TryFromWasmError)? as u64;
        let offset = (ptr >> 32) as u32;
        let length = (ptr & 0xFFFFFFFF) as u32;
        Ok(Self { offset, length })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wide_pointer() {
        let span = WasmSpan {
            offset: u32::MAX,
            length: u32::MAX - 1,
        };
        let ptr = span.to_wasm();
        assert_eq!(WasmSpan::try_from_wasm(&ptr).unwrap(), span);
    }
}
