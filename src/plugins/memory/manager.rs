use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use anyhow::Context;
use wasmtime::{AsContextMut, Instance, Memory, Store, StoreContextMut, TypedFunc};

use crate::plugins::StoreData;

use super::{WasmPtr, WasmSpan};

/// Controls sending and recieving data from the WebAssembly engine.
pub trait WasmMemoryManager<T: Send + Sync = StoreData>: Send + Sync {
    /// Create a buffer that can be accessed by the sandboxed WASM code
    /// and will be automatically deallocated when dropped.
    ///
    /// One could use this method to send values:
    ///
    /// 1. Allocate a buffer in WASM memory.
    /// 2. Copy data to the buffer
    /// 3. Send the buffer's pointer to WASM code
    fn alloc<'a>(&'a self, store: StoreContextMut<'a, T>, length: WasmPtr) -> WasmSlice<'a, T>;
    /// [`WasmMemoryManager::alloc`] and copy a slice into the newly created buffer
    fn copy<'a>(&'a self, store: StoreContextMut<'a, T>, buffer: &[u8]) -> WasmSlice<'a, T> {
        let mut slice = self.alloc(store, buffer.len() as WasmPtr);
        slice.as_mut().copy_from_slice(buffer);
        slice
    }
    /// Create a view into existing owned WASM memory that will automatically deallocate when dropped.
    fn get_from_raw<'a>(
        &'a self,
        store: StoreContextMut<'a, T>,
        span: WasmSpan,
    ) -> WasmSlice<'a, T>;
    /// Create a view into existing borrowed WASM memory that will not automatically deallocate when dropped
    fn view<'a>(&'a self, store: StoreContextMut<'a, T>, span: WasmSpan) -> WasmSlice<'a, T>;
    fn raw_memory(&self) -> &Memory;
    /// Deallocate memory at an address in the sandbox.
    /// Prefer using [`WasmMemoryManager::alloc`], which does this automatically
    /// on drop.
    fn dealloc<'a>(&'a self, store: StoreContextMut<'a, T>, ptr: WasmPtr);
}

/// Controls sending and recieving data from the WebAssembly engine.
/// This manager cooperates with the sandboxed allocator using
/// plugin-defined alloc and dealloc functions to keep it from accidentally
/// overwriting the memory we let the sandbox access. However, it does
/// not work if plugins do not define these functions.
pub struct CooperativeMemoryManager<T: Send + Sync> {
    memory: Memory,
    allocator: TypedFunc<WasmPtr, WasmPtr>,
    deallocator: TypedFunc<WasmPtr, ()>,
    _store_context: PhantomData<T>,
}

impl<T: Send + Sync> CooperativeMemoryManager<T> {
    pub fn new(mut store: &mut Store<StoreData>, instance: &Instance) -> anyhow::Result<Self> {
        Ok(Self {
            memory: instance
                .get_memory(&mut store, "memory")
                .context("WASM code must expose its memory")?,
            allocator: instance
                .get_typed_func(&mut store, "mem_alloc")
                .context("WASM code must expose a `mem_alloc` function")?,
            deallocator: instance
                .get_typed_func(&mut store, "mem_dealloc")
                .context("WASM code must expose a `mem_free` function")?,
            _store_context: PhantomData,
        })
    }
}

impl<T: Send + Sync> WasmMemoryManager<T> for CooperativeMemoryManager<T> {
    fn alloc<'a>(&'a self, mut store: StoreContextMut<'a, T>, length: WasmPtr) -> WasmSlice<'a, T> {
        let offset = self
            .allocator
            .call(&mut store, length)
            .expect("wasm memory allocator failed");
        WasmSlice {
            span: WasmSpan { offset, length },
            manager: self,
            store,
            owned: true,
        }
    }

    fn view<'a>(&'a self, store: StoreContextMut<'a, T>, span: WasmSpan) -> WasmSlice<'a, T> {
        WasmSlice {
            span,
            manager: self,
            store,
            owned: false,
        }
    }

    fn get_from_raw<'a>(
        &'a self,
        store: StoreContextMut<'a, T>,
        span: WasmSpan,
    ) -> WasmSlice<'a, T> {
        let mut slice = self.view(store, span);
        slice.owned = true;
        slice
    }

    fn raw_memory(&self) -> &Memory {
        &self.memory
    }

    fn dealloc<'a>(&'a self, mut store: StoreContextMut<'a, T>, ptr: WasmPtr) {
        self.deallocator
            .call(&mut store, ptr)
            .expect("wasm memory deallocator failed");
    }
}

/// A view into a slice of memory controlled by the WASM engine
pub struct WasmSlice<'a, T: Send + Sync> {
    span: WasmSpan,
    manager: &'a dyn WasmMemoryManager<T>,
    store: StoreContextMut<'a, T>,
    /// Controls if this memory will be deallocated when this struct is dropped
    pub owned: bool,
}

impl<'a, T: Send + Sync> WasmSlice<'a, T> {
    /// Prevent access to memory not within the bounds of this slice
    fn guard_bounds(&self, offset: WasmPtr) {
        if offset >= self.span.length {
            panic!(
                "attempt to access beyond the bounds of this slice: {offset} > {}",
                self.span.length
            );
        }
    }

    /// Get the memory span that this struct references
    #[must_use]
    pub fn into_raw(mut self) -> WasmSpan {
        self.owned = false;
        self.span.clone()
    }
}

impl<'a, T: Send + Sync> IndexMut<WasmPtr> for WasmSlice<'a, T> {
    fn index_mut(&mut self, index: WasmPtr) -> &mut Self::Output {
        self.guard_bounds(index);
        &mut self.manager.raw_memory().data_mut(&mut self.store)[index as usize]
    }
}

impl<'a, T: Send + Sync> Index<WasmPtr> for WasmSlice<'a, T> {
    type Output = u8;

    fn index(&self, index: WasmPtr) -> &Self::Output {
        self.guard_bounds(index);
        &self.manager.raw_memory().data(&self.store)[index as usize]
    }
}

impl<'a, T: Send + Sync> AsRef<[u8]> for WasmSlice<'a, T> {
    fn as_ref(&self) -> &[u8] {
        let end_ptr = self.span.offset + self.span.length;
        &self.manager.raw_memory().data(&self.store)[self.span.offset as usize..end_ptr as usize]
    }
}

impl<'a, T: Send + Sync> AsMut<[u8]> for WasmSlice<'a, T> {
    fn as_mut(&mut self) -> &mut [u8] {
        let end_ptr = self.span.offset + self.span.length;
        &mut self.manager.raw_memory().data_mut(&mut self.store)
            [self.span.offset as usize..end_ptr as usize]
    }
}

impl<'a, T: Send + Sync> ToString for WasmSlice<'a, T> {
    fn to_string(&self) -> String {
        String::from_utf8(self.as_ref().to_vec()).unwrap()
    }
}

impl<'a, T: Send + Sync> Drop for WasmSlice<'a, T> {
    fn drop(&mut self) {
        if self.owned {
            self.manager
                .dealloc(self.store.as_context_mut(), self.span.offset)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wasmtime::{AsContextMut, Engine, MemoryType, Store};

    fn bootstrap() -> (Store<MockState>, MockMemoryManger) {
        let engine = Engine::default();
        let mut store = Store::new(&engine, MockState::default());
        let memory = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();
        (store, MockMemoryManger { memory })
    }

    struct MockMemoryManger {
        memory: Memory,
    }

    #[derive(Default)]
    struct MockState {
        allocated_spans: HashMap<WasmPtr, WasmPtr>,
        memory_offset: WasmPtr,
    }

    impl WasmMemoryManager<MockState> for MockMemoryManger {
        fn alloc<'a>(
            &'a self,
            mut store: StoreContextMut<'a, MockState>,
            length: WasmPtr,
        ) -> WasmSlice<'a, MockState> {
            const PAGE_SIZE: u64 = 65536;
            let offset = store.data().memory_offset;
            if (offset + length) as u64 > PAGE_SIZE * self.memory.size(&store) {
                self.memory
                    .grow(&mut store, (length as u64).div_ceil(PAGE_SIZE))
                    .unwrap();
            }

            let span = WasmSpan {
                offset: store.data().memory_offset,
                length,
            };

            {
                let data = store.data_mut();
                data.allocated_spans.insert(offset, span.length);
                data.memory_offset += length;
            }

            WasmSlice {
                span,
                manager: self,
                store,
                owned: true,
            }
        }

        fn dealloc<'a>(&'a self, mut store: StoreContextMut<'a, MockState>, ptr: WasmPtr) {
            let data = store.data_mut();
            data.allocated_spans
                .remove(&ptr)
                .expect("Invalid address in dealloc");
        }

        fn raw_memory(&self) -> &Memory {
            &self.memory
        }

        fn get_from_raw<'a>(
            &'a self,
            store: StoreContextMut<'a, MockState>,
            span: WasmSpan,
        ) -> WasmSlice<'a, MockState> {
            let mut slice = self.view(store, span);
            slice.owned = true;
            slice
        }

        fn view<'a>(
            &'a self,
            store: StoreContextMut<'a, MockState>,
            span: WasmSpan,
        ) -> WasmSlice<'a, MockState> {
            WasmSlice {
                span,
                manager: self,
                store,
                owned: false,
            }
        }
    }

    #[test]
    fn test_slice_valid_access() {
        let (mut store, manager) = bootstrap();
        let mut slice = manager.alloc(store.as_context_mut(), 10);
        _ = slice[0];
        slice[0] = 1;
        _ = slice[9];
        slice[9] = 1;
    }

    #[test]
    #[should_panic]
    fn test_slice_invalid_access() {
        let (mut store, manager) = bootstrap();
        let slice = manager.alloc(store.as_context_mut(), 10);
        _ = slice[10];
    }

    #[test]
    fn test_slice_read_and_write() {
        let (mut store, manager) = bootstrap();
        let mut slice = manager.alloc(store.as_context_mut(), 2);
        slice[0] = 1;
        slice[1] = 2;
        assert_eq!(slice[0], 1);
        assert_eq!(slice[1], 2);
    }

    #[test]
    fn test_slice_to_string() {
        let (mut store, manager) = bootstrap();
        let mut slice = manager.alloc(store.as_context_mut(), 2);
        slice[0] = b'a';
        slice[1] = b'b';
        assert_eq!(slice.to_string(), "ab");
    }

    #[test]
    fn test_slice_into_raw() {
        let (mut store, manager) = bootstrap();
        let slice = manager.alloc(store.as_context_mut(), 2);
        let raw = slice.into_raw();
        assert_eq!(raw.offset, 0);
        assert_eq!(raw.length, 2);
    }

    #[test]
    fn test_slice_drop() {
        let (mut store, manager) = bootstrap();
        let slice = manager.alloc(store.as_context_mut(), 2);
        let raw = slice.into_raw();
        assert_eq!(store.data().allocated_spans.len(), 1);
        let slice = manager.get_from_raw(store.as_context_mut(), raw);
        drop(slice);
        assert_eq!(store.data().allocated_spans.len(), 0);
    }

    #[test]
    fn test_slice_drop_unowned() {
        let (mut store, manager) = bootstrap();
        let mut slice = manager.alloc(store.as_context_mut(), 2);
        slice.owned = false;
        drop(slice);
        assert_eq!(store.data().allocated_spans.len(), 1);
    }
}
