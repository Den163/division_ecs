use std::ptr::null;

use crate::archetype_data_page::ArchetypeDataPage;

use super::access::ComponentQueryAccess;

pub struct ComponentPageIter<'a, T: ComponentQueryAccess> {
    ptrs: T::PtrTuple<'a>,

    entity_ids: *const u32,
    next_entity_index: usize,
    entities_len: usize,
}

impl<'a, T: ComponentQueryAccess> ComponentPageIter<'a, T> {
    pub unsafe fn new<'b: 'a>(
        page: &'b ArchetypeDataPage,
        component_offsets: &T::OffsetTuple,
    ) -> Self {
        let ptrs = T::get_ptrs(page, component_offsets);

        Self {
            ptrs,
            entity_ids: page.entity_id_ptrs(),
            next_entity_index: 0,
            entities_len: page.entity_count(),
        }
    }

    pub fn empty() -> Self {
        Self {
            ptrs: T::null_ptrs::<'a>(),
            entity_ids: null(),
            next_entity_index: 0,
            entities_len: 0,
        }
    }

    pub unsafe fn current_entity_id(&self) -> u32 {
        *self.entity_ids.add(self.next_entity_index - 1)
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for ComponentPageIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_entity_index >= self.entities_len {
            return None;
        }

        let curr_entity_idx = self.next_entity_index;
        let ptrs = T::add_to_ptrs(&self.ptrs, curr_entity_idx);

        self.next_entity_index += 1;

        
        return Some(T::ptrs_to_refs(ptrs));
    }
}

impl<'a, T: ComponentQueryAccess> ExactSizeIterator for ComponentPageIter<'a, T> {
    fn len(&self) -> usize {
        self.entities_len
    }
}
