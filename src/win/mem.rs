
use windows::Win32::System::Memory::{GetProcessHeap, HeapAlloc, HeapFree, HEAP_FLAGS};

pub struct HeapMemory(pub *mut std::ffi::c_void);

impl HeapMemory {
    pub fn alloc(size: usize) -> Self {
        unsafe {
            Self(HeapAlloc(GetProcessHeap().expect("process heap"), HEAP_FLAGS(0), size))
        }
    }

    pub fn try_alloc(size: usize) -> anyhow::Result<Self> {
        let mem = Self::alloc(size);
        if mem.is_invalid() {
            Err(anyhow::anyhow!("Allocation failed"))
        } else {
            Ok(mem)
        }
    }

    pub fn into_ptr(self) -> *mut std::ffi::c_void {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

    pub fn is_invalid(&self) -> bool {
        self.0.is_null()
    }
}

impl Drop for HeapMemory {
    fn drop(&mut self) {
        unsafe {
            let _ = HeapFree(GetProcessHeap().expect("process heap"), HEAP_FLAGS(0), Some(self.0));
        }
    }
}
