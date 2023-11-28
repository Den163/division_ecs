use crate::mem_utils;
use std::alloc::Layout;

pub trait PtrMutReallocExt<T> {
    unsafe fn realloc(self, old_capacity: usize, new_capacity: usize) -> Self;
    unsafe fn realloc_with_uninit_capacity_zeroing(
        self,
        old_capacity: usize,
        new_capacity: usize,
    ) -> Self;
}

pub unsafe fn alloc<T>(capacity: usize) -> *mut T {
    std::alloc::alloc(mem_utils::layout_of::<T>(capacity)) as *mut T
}

pub unsafe fn alloc_zeroed<T>(capacity: usize) -> *mut T {
    std::alloc::alloc_zeroed(self::layout_of::<T>(capacity)) as *mut T
}

pub unsafe fn realloc<T>(
    ptr: *mut T,
    old_capacity: usize,
    new_capacity: usize,
) -> *mut T {
    std::alloc::realloc(
        ptr as *mut u8,
        mem_utils::layout_of::<T>(old_capacity),
        new_capacity,
    ) as *mut T
}

pub unsafe fn realloc_with_uninit_capacity_zeroing<T>(
    ptr: *mut T,
    old_capacity: usize,
    new_capacity: usize,
) -> *mut T {
    let new_ptr = alloc_zeroed(new_capacity);
    ptr.copy_to_nonoverlapping(new_ptr, old_capacity);

    dealloc(ptr, old_capacity);

    new_ptr
}

pub unsafe fn dealloc<T>(ptr: *mut T, capacity: usize) {
    std::alloc::dealloc(ptr as *mut u8, mem_utils::layout_of::<T>(capacity))
}

pub unsafe fn layout_of<T>(capacity: usize) -> Layout {
    Layout::from_size_align_unchecked(
        std::mem::size_of::<T>() * capacity,
        std::mem::align_of::<T>(),
    )
}

impl<T> PtrMutReallocExt<T> for *mut T {
    unsafe fn realloc(mut self, old_capacity: usize, new_capacity: usize) -> *mut T {
        self = mem_utils::realloc(self, old_capacity, new_capacity);
        self
    }

    unsafe fn realloc_with_uninit_capacity_zeroing(
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

pub unsafe fn alloc_bit_vec(len: usize) -> *mut u32 {
    mem_utils::alloc_zeroed(get_bitvec_size(len))
}

pub unsafe fn dealloc_bit_vec(ptr: *mut u32, len: usize) {
    dealloc(ptr, get_bitvec_size(len))
}

pub unsafe fn realloc_bit_vec(ptr: *mut u32, old_len: usize, new_len: usize) -> *mut u32 {
    let old_size = get_bitvec_size(old_len);
    let new_size = get_bitvec_size(new_len);

    if old_size == new_size {
        return ptr;
    }

    let new_ptr = alloc(new_size);
    ptr.copy_to_nonoverlapping(new_ptr, old_size);
    new_ptr.add(old_size).write_bytes(0, new_size - old_size);

    dealloc(ptr, old_size);
    new_ptr
}

pub unsafe fn toggle_bitvec_bit(ptr: *mut u32, bit_index: usize) {
    let base_bits = get_bitvec_base_bits();
    let mask_index = bit_index / base_bits;
    let mask_bit = bit_index % base_bits;
    let alive_mask = 1 >> mask_bit;

    let mask_ptr = ptr.add(mask_index as usize);
    *mask_ptr ^= alive_mask;
}

pub unsafe fn is_bitvec_bit_on(ptr: *const u32, bit_index: usize) -> bool {
    let base_bits = get_bitvec_base_bits();
    let mask_index = bit_index / base_bits;
    let mask_bit = bit_index % base_bits;
    let alive_mask = 1 >> mask_bit;

    let mask_ptr = ptr.add(mask_index as usize);
    (*mask_ptr & alive_mask) == alive_mask
}

#[inline]
pub fn get_bitvec_size(elements: usize) -> usize {
    elements / get_bitvec_base_bits() + 1
}

const fn get_bitvec_base_bits() -> usize {
    std::mem::size_of::<u32>() * 8
}
