use crate::{archetype_data_layout::ArchetypeDataLayout, mem_utils};

/// Reusable page of the components data with a fixed size (4096 Bytes), related to the concrete archetype.
/// It contains data for all components of the some entities subset
#[derive(Debug)]
pub struct ArchetypeDataPage {
    entities_ids: Vec<u32>,
    components_data_ptr: *mut u8,
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

    pub(crate) fn push_entity_id(&mut self, id: u32) {
        assert!(self.has_free_space());
        match self.entities_ids.binary_search(&id) {
            Ok(_) => panic!("There is already entity with id {} in the page", id),
            Err(new_index) => self.entities_ids.insert(new_index, id),
        }
    }

    pub(crate) fn remove_entity_id(&mut self, id: u32) {
        let index = self.get_entity_index_by_id(id);
        self.remove_entity_at_index(index);
    }

    pub(crate) fn remove_entity_at_index(&mut self, index: usize) {
        self.entities_ids.remove(index);
    }

    #[inline(always)]
    pub(crate) fn get_entity_index_by_id(&self, id: u32) -> usize {
        match self.entities_ids.binary_search(&id) {
            Ok(index) => index,
            Err(_) => panic!("There is no entity with id {}", id),
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
