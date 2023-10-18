use crate::mem_utils;
use std::alloc::Layout;

pub trait PtrMutReallocExt<T> {
    fn realloc(self, old_capacity: usize, new_capacity: usize) -> Self;
    fn realloc_with_uninit_capacity_zeroing(
        self,
        old_capacity: usize,
        new_capacity: usize,
    ) -> Self;
}

pub fn alloc<T>(capacity: usize) -> *mut T {
    unsafe { std::alloc::alloc(mem_utils::layout_of::<T>(capacity)) as *mut T }
}

pub fn alloc_zeroed<T>(capacity: usize) -> *mut T {
    unsafe { std::alloc::alloc_zeroed(self::layout_of::<T>(capacity)) as *mut T }
}

pub fn realloc<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    unsafe {
        std::alloc::realloc(
            ptr as *mut u8,
            mem_utils::layout_of::<T>(old_capacity),
            new_capacity,
        ) as *mut T
    }
}

pub fn realloc_with_uninit_capacity_zeroing<T>(
    ptr: *mut T,
    old_capacity: usize,
    new_capacity: usize,
) -> *mut T {
    let new_ptr = alloc_zeroed(new_capacity);
    unsafe {
        ptr.copy_to_nonoverlapping(new_ptr, old_capacity);
    }
    dealloc(ptr, old_capacity);

    new_ptr
}

pub fn dealloc<T>(ptr: *mut T, capacity: usize) {
    unsafe { std::alloc::dealloc(ptr as *mut u8, mem_utils::layout_of::<T>(capacity)) }
}

pub fn layout_of<T>(capacity: usize) -> Layout {
    unsafe {
        Layout::from_size_align_unchecked(
            std::mem::size_of::<T>() * capacity,
            std::mem::align_of::<T>(),
        )
    }
}

impl<T> PtrMutReallocExt<T> for *mut T {
    fn realloc(mut self, old_capacity: usize, new_capacity: usize) -> *mut T {
        self = mem_utils::realloc(self, old_capacity, new_capacity);
        self
    }

    fn realloc_with_uninit_capacity_zeroing(
        mut self,
        old_capacity: usize,
        new_capacity: usize,
    ) -> *mut T {
        self = mem_utils::realloc_with_uninit_capacity_zeroing(
            self,
            old_capacity,
            new_capacity,
        );
        self
    }
}
