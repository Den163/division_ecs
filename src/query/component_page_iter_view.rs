use std::ptr::null;

use super::access::ComponentQueryAccess;

pub struct ComponentPageIterView<T: ComponentQueryAccess> {
    pub ptrs: T::PtrTuple,
    pub entity_ids: *const u32,
    pub entity_count: usize,
}

impl<T: ComponentQueryAccess> ComponentPageIterView<T> {
    pub fn empty() -> Self {
        Self {
            ptrs: T::null_ptrs(),
            entity_ids: null(),
            entity_count: 0,
        }
    }
}

impl<T: ComponentQueryAccess> Clone for ComponentPageIterView<T> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs,
            entity_ids: self.entity_ids,
            entity_count: self.entity_count,
        }
    }
}

impl<T: ComponentQueryAccess> Copy for ComponentPageIterView<T> {

}
