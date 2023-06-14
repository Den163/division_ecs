use std::{any::TypeId, hash::{Hash, Hasher}};

use crate::{component_type::{ComponentType}, mem_utils};

#[derive(Debug)]
pub struct Archetype {
    ids: *mut TypeId,
    sizes: *mut usize,
    aligns: *mut usize,
    component_count: usize,
}

impl Archetype {
    pub(crate) fn new(components: &mut [ComponentType]) -> Self {
        let component_count = components.len();
        debug_assert!(component_count > 0);
        
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

    pub fn find_component_index(&self, type_id: TypeId) -> Option<usize> {
        unsafe {
            let slice = & *std::ptr::slice_from_raw_parts(self.ids, self.component_count);
            match slice.binary_search(&type_id) {
                Ok(idx) => Some(idx),
                Err(_) => None
            }
        }
    }

    #[inline(always)]
    pub fn component_count(&self) -> usize { 
        self.component_count
    }

    pub fn is_same_as(&self, other: &Self) -> bool {
        if self.component_count != other.component_count {
            return false;
        }

        for i in 0..self.component_count {
            let is_different_id = unsafe { *self.ids.add(i) != *other.ids.add(i) };
            if is_different_id  {
                return false;
            }
        }

        return true;
    }

    pub fn is_extends(&self, other: &Self) -> bool {
        if self.component_count < other.component_count {
            return false;
        }

        let (self_ids, other_ids) = unsafe {(
            std::slice::from_raw_parts(self.ids, self.component_count),
            std::slice::from_raw_parts(other.ids, other.component_count)
        )};

        let mut left_idx = 0;
        for id in other_ids {
            let result  = self_ids[left_idx..].binary_search(id);
            match result {
                Ok(found_idx) => {
                    left_idx = found_idx + 1;
                    continue 
                },
                Err(_) => return false,
            }
        }

        return true;
    }
}

impl Clone for Archetype {
    fn clone(&self) -> Self {
        let component_count = self.component_count;
        let ids = mem_utils::alloc(component_count);
        let sizes = mem_utils::alloc(component_count);
        let aligns = mem_utils::alloc(component_count);
        unsafe {
            self.ids.copy_to_nonoverlapping(ids, component_count);
            self.sizes.copy_to_nonoverlapping(sizes, component_count);
            self.aligns.copy_to_nonoverlapping(aligns, component_count);
        }

        Self { 
            ids,
            sizes,
            aligns, 
            component_count
        }
    }
}

impl Hash for Archetype {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in 0..self.component_count {
            let id = unsafe {
                *self.ids.add(i)
            };
            
            id.hash(state);
        }
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        mem_utils::dealloc(self.ids, self.component_count);
        mem_utils::dealloc(self.sizes, self.component_count);
        mem_utils::dealloc(self.aligns, self.component_count);
    }
}