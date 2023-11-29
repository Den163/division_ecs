use crate::{archetype_data_layout::ArchetypeDataLayout, mem_utils};

/// Reusable page of the components data with a fixed size (4096 Bytes), related to the concrete archetype.
/// It contains data for all components of the some entities subset
#[derive(Debug)]
pub struct ArchetypeDataPage {
    entities_ids: Vec<u32>,
    components_data_ptr: *mut u8,
}

pub(crate) struct SwapRemoveInfo {
    pub swapped_id: u32,
    pub swapped_index: usize,
}

impl ArchetypeDataPage {
    pub const PAGE_SIZE_BYTES: usize = 4096 * 4;

    pub(crate) fn new() -> Self {
        let components_data_ptr = unsafe { mem_utils::alloc(Self::PAGE_SIZE_BYTES) };

        ArchetypeDataPage {
            components_data_ptr,
            entities_ids: Vec::new(),
        }
    }

    pub(crate) fn set_layout(&mut self, layout: &ArchetypeDataLayout) {
        let capacity = layout.entities_capacity();
        self.entities_ids.reserve(capacity);
    }

    #[inline(always)]
    pub(crate) fn entities_count(&self) -> usize {
        self.entities_ids.len()
    }

    #[inline(always)]
    pub(crate) fn entities_capacity(&self) -> usize {
        self.entities_ids.capacity()
    }

    #[inline(always)]
    pub(crate) fn entities_ids<'a>(&'a self) -> &'a [u32] {
        &self.entities_ids
    }

    #[inline(always)]
    pub(crate) fn has_free_space(&self) -> bool {
        self.entities_count() < self.entities_capacity()
    }

    pub(crate) fn add_entity_id(&mut self, id: u32) -> usize {
        assert!(self.has_free_space());
        self.entities_ids.push(id);
        self.entities_ids.len() - 1
    }

    pub(crate) fn swap_remove_entity_at_index(
        &mut self,
        index: usize,
    ) -> Option<SwapRemoveInfo> {
        self.entities_ids.swap_remove(index);
        if index < self.entities_ids.len() {
            Some(SwapRemoveInfo {
                swapped_id: self.entities_ids[index],
                swapped_index: self.entities_ids.len(),
            })
        } else {
            None
        }
    }

    #[inline(always)]
    pub(crate) fn get_component_data_ptr_mut(
        &self,
        entity_index: usize,
        component_offset: usize,
        type_size: usize,
    ) -> *mut u8 {
        unsafe {
            self.components_data_ptr
                .add(component_offset + entity_index * type_size)
        }
    }

    #[inline(always)]
    pub(crate) fn get_component_data_ptr(
        &self,
        entity_index: usize,
        component_offset: usize,
        type_size: usize,
    ) -> *const u8 {
        unsafe {
            self.components_data_ptr
                .add(component_offset + entity_index * type_size)
        }
    }
}

impl Drop for ArchetypeDataPage {
    fn drop(&mut self) {
        unsafe { mem_utils::dealloc(self.components_data_ptr, Self::PAGE_SIZE_BYTES) };
    }
}
