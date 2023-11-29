use std::ptr::null_mut;

use crate::{archetype::Archetype, archetype_data_page::ArchetypeDataPage, mem_utils};

#[derive(Debug)]
pub(crate) struct ArchetypeDataLayout {
    component_offsets_ptr: *mut usize,
    component_count: usize,
    entities_capacity: usize,
}

impl ArchetypeDataLayout {
    pub fn new(archetype: &Archetype) -> Self {
        let component_count = archetype.component_count();

        let ptr_size = std::mem::size_of::<usize>();
        let max_align = archetype
            .components_iter()
            .map(|c| c.align())
            .max()
            .unwrap();
        let max_align = max_align % ptr_size;

        let bytes_per_components_row_approx =
            ArchetypeDataPage::PAGE_SIZE_BYTES / component_count - max_align;
        let entities_capacity = archetype
            .components_iter()
            .map(|c| bytes_per_components_row_approx / c.size())
            .min()
            .unwrap();

        let component_offsets: *mut usize = unsafe { mem_utils::alloc(component_count) };
        let mut offset = 0;
        for (i, ct) in archetype.components_iter().enumerate() {
            let align_offset = offset % ct.align();

            if align_offset != 0 {
                offset += ct.align() - align_offset;
            }
            unsafe {
                component_offsets.add(i).write(offset);
            }

            offset += ct.size() * entities_capacity;
        }

        assert!(offset <= ArchetypeDataPage::PAGE_SIZE_BYTES);

        ArchetypeDataLayout {
            component_offsets_ptr: component_offsets,
            component_count,
            entities_capacity,
        }
    }

    pub fn empty() -> ArchetypeDataLayout {
        ArchetypeDataLayout {
            component_offsets_ptr: null_mut(),
            component_count: 0,
            entities_capacity: 0
        }
    }

    #[inline(always)]
    pub fn component_offsets(&self) -> *const usize {
        self.component_offsets_ptr
    }

    #[inline(always)]
    pub fn entities_capacity(&self) -> usize {
        self.entities_capacity
    }
}

impl Drop for ArchetypeDataLayout {
    fn drop(&mut self) {
        unsafe {
            mem_utils::dealloc(self.component_offsets_ptr, self.component_count);
        }
    }
}
