
use windows::Win32::System::Memory::{GetProcessHeap, HeapAlloc, HeapFree, HEAP_FLAGS};

pub struct HeapMemory(*mut std::ffi::c_void);

impl HeapMemory {
    /// Create memory allocation, returning an error if allocation failed
    pub fn alloc(size: usize) -> anyhow::Result<Self> {
        let mem = unsafe {
            HeapAlloc(GetProcessHeap().expect("process heap"), HEAP_FLAGS(0), size)
        };
        if mem.is_null() {
            Err(anyhow::anyhow!("Allocation failed"))
        } else {
            Ok(Self(mem))
        }
    }

    /// Convert from pointer
    /// 
    /// # Safety
    /// Pointer must have been previously allocated via HeapMemory
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Convert into pointer, forgetting pointer was allocated
    pub fn into_ptr(self) -> *mut std::ffi::c_void {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

    /// Convert into pointer, without forgetting pointer was allocated
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }
}

impl Drop for HeapMemory {
    fn drop(&mut self) {
        unsafe {
            let _ = HeapFree(GetProcessHeap().expect("process heap"), HEAP_FLAGS(0), Some(self.0));
        }
    }
}
