use std::alloc::Layout;
use crate::mem_utils;

pub fn alloc<T>(capacity: usize) -> *mut T {
    unsafe {
        std::alloc::alloc(mem_utils::layout_of::<T>(capacity)) as *mut T
    }
}

pub fn alloc_zeroed<T>(capacity: usize) -> *mut T {
    unsafe {
        std::alloc::alloc_zeroed(self::layout_of::<T>(capacity)) as *mut T
    }
}

pub fn dealloc<T>(ptr: *mut T, capacity: usize) {
    unsafe {
        std::alloc::dealloc(ptr as *mut u8, mem_utils::layout_of::<T>(capacity))
    }
}

pub fn layout_of<T>(capacity: usize) -> Layout {
    unsafe {
        Layout::from_size_align_unchecked(
            std::mem::size_of::<T>() * capacity,
            std::mem::align_of::<T>()
        )
    }
}