pub mod manager;

/// The address of a byte that may be controlled by the WASM engine.
pub type WasmPtr = u32;

/// A span of bytes that may be controlled by the WASM engine.
/// This can be passed to a [`WasmMemoryManager`]
/// to be read or written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmSpan {
    pub offset: WasmPtr,
    pub length: WasmPtr,
}

impl WasmSpan {
    /// Returns an i64 where the first half is the offset and the 2nd is the length
    pub fn as_wide_pointer(&self) -> i64 {
        let ptr = (self.offset as u64) << 32 | (self.length as u64);
        ptr as i64
    }

    /// Parses an i64 where the first half is the offset and the 2nd is the length
    pub fn from_wide_pointer(ptr: i64) -> Self {
        let offset = (ptr >> 32) as u32;
        let length = (ptr & 0xFFFFFFFF) as u32;
        Self { offset, length }
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
        let ptr = span.as_wide_pointer();
        assert_eq!(WasmSpan::from_wide_pointer(ptr), span);
    }
}
