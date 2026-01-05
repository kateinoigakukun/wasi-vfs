//! A global allocator which wraps wasi-libc's malloc while targeting wasm32-unknown-unknown

#[cfg(not(feature = "module-linking"))]
const MIN_ALIGN: usize = 8;

#[cfg(not(feature = "module-linking"))]
struct WasiAllocator;

#[cfg(not(feature = "module-linking"))]
unsafe impl std::alloc::GlobalAlloc for WasiAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        unsafe extern "C" {
            fn malloc(amt: usize) -> *mut std::ffi::c_void;
            fn aligned_alloc(a: usize, b: usize) -> *mut std::ffi::c_void;
        }
        unsafe {
            if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
                malloc(layout.size()) as *mut u8
            } else {
                aligned_alloc(layout.align(), layout.size()) as *mut u8
            }
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
        unsafe extern "C" {
            fn calloc(amt: usize, amt2: usize) -> *mut std::ffi::c_void;
        }
        unsafe {
            if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
                calloc(layout.size(), 1) as *mut u8
            } else {
                let ptr = self.alloc(layout);
                if !ptr.is_null() {
                    std::ptr::write_bytes(ptr, 0, layout.size());
                }
                ptr
            }
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: std::alloc::Layout) {
        unsafe extern "C" {
            fn free(ptr: *mut std::ffi::c_void);
        }
        unsafe {
            free(ptr as *mut std::ffi::c_void)
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        unsafe extern "C" {
            fn realloc(ptr: *mut std::ffi::c_void, amt: usize) -> *mut std::ffi::c_void;
        }
        unsafe {
            if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
                realloc(ptr as *mut std::ffi::c_void, new_size) as *mut u8
            } else {
                let new_layout =
                    std::alloc::Layout::from_size_align_unchecked(new_size, layout.align());

                let new_ptr = std::alloc::GlobalAlloc::alloc(self, new_layout);
                if !new_ptr.is_null() {
                    let size = std::cmp::min(layout.size(), new_size);
                    std::ptr::copy_nonoverlapping(ptr, new_ptr, size);
                    std::alloc::GlobalAlloc::dealloc(self, ptr, layout);
                }
                new_ptr
            }
        }
    }
}

#[cfg(not(feature = "module-linking"))]
#[global_allocator]
static GLOBAL: WasiAllocator = WasiAllocator;

// use self-contained allocator when module-linking
#[cfg(feature = "module-linking")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
