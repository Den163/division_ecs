use crate::{Archetype, archetype_data_page::ArchetypeDataPage, mem_utils};

#[derive(Debug)]
pub struct ArchetypeLayout {
    offsets: *mut usize,
    component_count: usize,
    entities_capacity: usize,
}

impl ArchetypeLayout {
    pub fn new(source_archetype: &Archetype) -> ArchetypeLayout {
        let sizes = source_archetype.component_sizes();
        let aligns = source_archetype.component_aligns();
        let component_count = source_archetype.component_count();

        let entities_capacity = unsafe {
            Self::calculate_entities_capacity(sizes, aligns, component_count)
        };
        let offsets = unsafe {
            Self::calculate_offsets(sizes, aligns, component_count, entities_capacity)
        };

        Self {
            offsets,
            component_count,
            entities_capacity
        }
    }

    unsafe fn calculate_entities_capacity(
        sizes: *const usize,
        aligns: *const usize,
        component_count: usize,
    ) -> usize {
        let ptr_size = std::mem::size_of::<usize>();

        let aligns = std::slice::from_raw_parts(aligns, component_count);
        let max_align = aligns.into_iter().max().unwrap();
        let max_align = max_align % ptr_size;

        let bytes_per_components_row_approx =
            ArchetypeDataPage::PAGE_SIZE_BYTES / component_count - max_align;

        let sizes = std::slice::from_raw_parts(sizes, component_count);
        let entities_capacity = sizes
            .iter()
            .map(|s| bytes_per_components_row_approx / *s)
            .min()
            .unwrap();

        entities_capacity
    }

    unsafe fn calculate_offsets(
        sizes: *const usize,
        aligns: *const usize,
        component_count: usize,
        entities_capacity: usize,
    ) -> *mut usize {
        let component_offsets: *mut usize = mem_utils::alloc(component_count);
        let mut offset = 0;

        for i in 0..component_count {
            let size = *sizes.add(i);
            let align = *aligns.add(i);
            let align_offset = offset % align;

            if align_offset != 0 {
                offset += align - align_offset;
            }

            *component_offsets.add(i) = offset;

            offset += size * entities_capacity;
        }

        assert!(offset <= ArchetypeDataPage::PAGE_SIZE_BYTES);
        component_offsets
    }

    #[inline]
    pub fn entities_capacity(&self) -> usize {
        self.entities_capacity
    }

    #[inline]
    pub fn component_offsets(&self) -> *const usize {
        self.offsets
    }
}

impl Drop for ArchetypeLayout {
    fn drop(&mut self) {
        unsafe {
            mem_utils::dealloc(self.offsets, self.component_count);
        }
    }
}