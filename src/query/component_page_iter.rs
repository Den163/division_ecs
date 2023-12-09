use std::marker::PhantomData;

use super::{access::ComponentQueryAccess, component_page_iter_view::ComponentPageIterView};

pub struct ComponentPageIter<'a, T: ComponentQueryAccess> {
    view: ComponentPageIterView<T>,
    next_entity_index: usize,

    _lifetime: PhantomData<&'a T>
}

impl<'a, T: ComponentQueryAccess> ComponentPageIter<'a, T> {
    #[inline]
    pub(crate) fn new(
        view: ComponentPageIterView<T>
    ) -> Self {
        Self {
            view,
            next_entity_index: 0,

            _lifetime: PhantomData::default()
        }
    }

    #[inline]
    pub fn empty() -> Self {
        Self::new(ComponentPageIterView::empty())
    }

    pub unsafe fn current_entity_id(&self) -> u32 {
        *self.view.entity_ids.add(self.next_entity_index - 1)
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for ComponentPageIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_entity_index >= self.view.entity_count {
            return None;
        }

        let curr_entity_idx = self.next_entity_index;
        let ptrs = T::add_to_ptrs(&self.view.ptrs, curr_entity_idx);

        self.next_entity_index += 1;

        
        return Some(T::ptrs_to_refs(ptrs));
    }
}

impl<'a, T: ComponentQueryAccess> ExactSizeIterator for ComponentPageIter<'a, T> {
    fn len(&self) -> usize {
        self.view.entity_count
    }
}
