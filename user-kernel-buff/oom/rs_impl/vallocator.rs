use alloc::alloc::{Layout, Allocator, AllocError};
use core::ptr::NonNull;
use kernel::bindings;
pub(crate) struct VAllocator;

unsafe impl Allocator for VAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let raw_ptr: *mut u8 = bindings::vmalloc_user(layout.size() as _) as _;
            let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
            Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        unsafe { bindings::vfree(ptr.as_ptr() as _); }
    }
}
