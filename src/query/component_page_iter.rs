use std::{marker::PhantomData, ptr::null};

use crate::archetype_data_page::ArchetypeDataPage;

use super::access::ComponentQueryAccess;

pub struct ComponentPageIter<'a, T: ComponentQueryAccess> {
    page_ptr: *const ArchetypeDataPage,
    component_offsets: T::OffsetsTuple,
    next_entity_index: usize,
    entities_len: usize,

    phantom_data: PhantomData<&'a T>,
}

impl<'a, T: ComponentQueryAccess> ComponentPageIter<'a, T> {
    pub unsafe fn new(
        page_ptr: *const ArchetypeDataPage,
        component_offsets: T::OffsetsTuple,
    ) -> Self {
        Self {
            page_ptr,
            component_offsets,
            next_entity_index: 0,
            entities_len: (&*page_ptr).entities_count(),
            phantom_data: PhantomData::default(),
        }
    }

    pub fn empty() -> Self {
        Self {
            page_ptr: null(),
            component_offsets: T::OffsetsTuple::default(),
            next_entity_index: 0,
            entities_len: 0,
            phantom_data: PhantomData::default(),
        }
    }

    pub unsafe fn current_entity_id(&self) -> u32 {
        let page = &*self.page_ptr;
        *page
            .entities_ids()
            .get_unchecked(self.next_entity_index - 1)
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for ComponentPageIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_entity_index >= self.entities_len {
            return None;
        }

        let page = unsafe { &*self.page_ptr };
        let curr_entity_idx = self.next_entity_index;

        self.next_entity_index += 1;

        return Some(T::get_refs(page, curr_entity_idx, &self.component_offsets));
    }
}

impl<'a, T: ComponentQueryAccess> ExactSizeIterator for ComponentPageIter<'a, T> {
    fn len(&self) -> usize {
        self.entities_len
    }
}
