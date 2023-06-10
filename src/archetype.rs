use std::{any::TypeId};

use crate::{component_type::{ComponentType}, mem_utils};

#[derive(Debug)]
pub struct Archetype {
    ids: *const TypeId,
    sizes: *const usize,
    aligns: *const usize,
    component_count: usize
}

impl Archetype {
    pub(crate) fn new(components: &[ComponentType]) -> Self {
        let component_count = components.len();
        debug_assert!(component_count > 0);

        let mut component_types = components.to_vec();
        component_types.sort_by_key(|c| { c.id() });
        
        let ids: *mut TypeId = mem_utils::alloc(component_count);
        let sizes: *mut usize = mem_utils::alloc(component_count);
        let aligns: *mut usize = mem_utils::alloc(component_count);

        for (i, comp) in components.iter().enumerate() {
            unsafe {
                *ids.add(i) = comp.id();
                *sizes.add(i) = comp.size();
                *aligns.add(i) = comp.align();
            };
        }

        Archetype { ids, sizes, aligns, component_count }
    }

    #[inline(always)]
    pub fn has_component<T: 'static>(&self) -> bool {
        self.find_component_index(TypeId::of::<T>()).is_some()
    }

    #[inline(always)]
    pub fn components_iter<'a>(&'a self) -> impl Iterator<Item=ComponentType> + 'a {
        (0..self.component_count).into_iter().map(|i| { unsafe {
            ComponentType::new(
                *self.ids.add(i), 
                *self.sizes.add(i), 
                *self.aligns.add(i) 
            ) 
        }})
    }

    pub(crate) fn find_component_index(&self, type_id: TypeId) -> Option<usize> {
        for i in 0..self.component_count {
            let id = unsafe { *self.ids.add(i) };
            if id == type_id {
                return Some(i);
            }
        }

        return None;
    }

    #[inline(always)]
    pub fn component_count(&self) -> usize { 
        self.component_count
    }
}