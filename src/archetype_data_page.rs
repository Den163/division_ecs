use crate::{mem_utils, Archetype};

/// Reusable page of the components data with a fixed size (4096 Bytes), related to the concrete archetype.
/// It contains data for all components of the some entities subset
#[derive(Debug)]
pub struct ArchetypeDataPage {
    entities_ids: Vec<u32>,
    components_data_ptr: *mut u8,
}

pub(crate) struct SwapRemoveInfo {
    pub id_to_replace: u32,
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

    pub(crate) fn set_archetype(&mut self, archetype: &Archetype) {
        let capacity = archetype.entities_capacity();
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
        debug_assert!(self.has_free_space());
        self.entities_ids.push(id);
        self.entities_ids.len() - 1
    }

    pub(crate) fn swap_remove_entity_at_index(
        &mut self,
        index: usize,
        archetype: &Archetype,
    ) -> Option<SwapRemoveInfo> {
        self.entities_ids.swap_remove(index);
        let last_swapped_index = self.entities_ids.len();
        if index < last_swapped_index {
            unsafe {
                self.move_component_data(last_swapped_index, index, archetype);
            }

            Some(SwapRemoveInfo {
                id_to_replace: self.entities_ids[index],
            })
        } else {
            None
        }
    }

    #[inline]
    unsafe fn move_component_data(
        &mut self,
        src_entity_index: usize,
        dst_entity_index: usize,
        archetype: &Archetype,
    ) {
        let sizes = archetype.component_sizes();
        let offsets = archetype.component_offsets();

        for i in 0..archetype.component_count() {
            let s = *sizes.add(i);
            let o = *offsets.add(i);

            let src_comp = self.get_component_data_ptr(src_entity_index, o, s);
            let dst_comp = self.get_component_data_ptr_mut(dst_entity_index, o, s);
            src_comp.copy_to_nonoverlapping(dst_comp, s);
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
