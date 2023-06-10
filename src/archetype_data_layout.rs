use crate::{archetype_data_page::ArchetypeDataPage, component_type::ComponentType, mem_utils};

pub(crate) struct ArchetypeDataLayout {
    component_offsets: *mut usize,
    component_count: usize,
    entities_capacity: usize
}

impl ArchetypeDataLayout {
    pub fn new(components: &[ComponentType]) -> Self {
        let component_count = components.len();

        let max_align = components.iter()
            .map(|c| { c.align() })
            .max().unwrap();
        let bytes_per_components_row_approx = ArchetypeDataPage::PAGE_SIZE_BYTES / component_count - max_align;
        let entities_capacity = components.iter()
            .map(|c| bytes_per_components_row_approx / c.size())
            .max().unwrap();

        let component_offsets: *mut usize = mem_utils::alloc(component_count);
        let mut offset = 0;
        for (i, ct) in components.iter().enumerate() {
            let align_offset = offset % ct.align();

            if align_offset != 0 {
                offset += ct.align() - align_offset;
            }
            unsafe {
                component_offsets.add(i).write(offset);
            }

            offset += ct.size() * entities_capacity;
        }

        debug_assert!(offset < ArchetypeDataPage::PAGE_SIZE_BYTES);

        ArchetypeDataLayout {
            component_offsets,
            component_count,
            entities_capacity
        }
    }
}