use crate::mem_utils;

pub unsafe fn alloc(len: usize) -> *mut u32 {
    mem_utils::alloc_zeroed(get_len(len))
}

pub unsafe fn dealloc(ptr: *mut u32, len: usize) {
    mem_utils::dealloc(ptr, get_len(len))
}

pub unsafe fn realloc(ptr: *mut u32, old_len: usize, new_len: usize) -> *mut u32 {
    let old_size = get_len(old_len);
    let new_size = get_len(new_len);

    if old_size == new_size {
        return ptr;
    }

    let new_ptr = mem_utils::alloc_zeroed(new_size);
    ptr.copy_to_nonoverlapping(new_ptr, old_size);

    mem_utils::dealloc(ptr, old_size);
    new_ptr
}

pub unsafe fn toggle_bit(ptr: *mut u32, bit_index: usize) {
    let base_bits = get_bitvec_base_bits();
    let mask_index = bit_index / base_bits;
    let mask_bit = bit_index % base_bits;
    let alive_mask = 1 >> mask_bit;

    let mask_ptr = ptr.add(mask_index as usize);
    *mask_ptr ^= alive_mask;
}

pub unsafe fn is_bit_on(ptr: *const u32, bit_index: usize) -> bool {
    let base_bits = get_bitvec_base_bits();
    let mask_index = bit_index / base_bits;
    let mask_bit = bit_index % base_bits;
    let alive_mask = 1 >> mask_bit;

    let mask_ptr = ptr.add(mask_index as usize);
    (*mask_ptr & alive_mask) == alive_mask
}

#[inline]
pub fn get_len(elements: usize) -> usize {
    elements / get_bitvec_base_bits() + 1
}

const fn get_bitvec_base_bits() -> usize {
    std::mem::size_of::<u32>() * 8
}
