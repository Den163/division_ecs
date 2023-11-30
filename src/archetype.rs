use std::{
    any::TypeId,
    hash::{Hash, Hasher}, ptr::null_mut,
};

use crate::{component_type::ComponentType, mem_utils, tuple::ComponentsTuple};

#[derive(Debug)]
pub struct Archetype {
    ids: *mut TypeId,
    sizes: *mut usize,
    aligns: *mut usize,
    component_count: usize,
}

pub trait ComponentTupleToArchetype: ComponentsTuple {
    fn into_archetype() -> Archetype;
}

impl Archetype {
    pub fn with_components<T: ComponentTupleToArchetype>() -> Self {
        T::into_archetype()
    }

    pub(crate) fn new(sorted_components: &[ComponentType]) -> Self {
        let component_count = sorted_components.len();
        assert!(component_count > 0);

        let (ids, sizes, aligns): (*mut TypeId, *mut usize, *mut usize) = unsafe {
            (
                mem_utils::alloc(component_count),
                mem_utils::alloc(component_count),
                mem_utils::alloc(component_count),
            )
        };

        for (i, comp) in sorted_components.iter().enumerate() {
            unsafe {
                *ids.add(i) = comp.id();
                *sizes.add(i) = comp.size();
                *aligns.add(i) = comp.align();
            };
        }

        Archetype {
            ids,
            sizes,
            aligns,
            component_count,
        }
    }

    pub(crate) fn empty() -> Archetype {
        Archetype {
            ids: null_mut(),
            sizes: null_mut(),
            aligns: null_mut(),
            component_count: 0
        }
    }

    #[inline(always)]
    pub fn has_component<T: 'static>(&self) -> bool {
        self.find_component_index(TypeId::of::<T>()).is_some()
    }

    #[inline(always)]
    pub fn components_iter<'a>(&'a self) -> impl Iterator<Item = ComponentType> + 'a {
        (0..self.component_count).into_iter().map(|i| unsafe {
            ComponentType::new(*self.ids.add(i), *self.sizes.add(i), *self.aligns.add(i))
        })
    }

    pub fn find_component_index_typed_checked<T: 'static>(&self) -> usize {
        let index = self.find_component_index(std::any::TypeId::of::<T>());
        match index {
            Some(i) => i,
            None => {
                let type_name = std::any::type_name::<T>();
                panic!("There is no type {type_name} in the archetype");
            }
        }
    }

    pub fn find_component_index(&self, type_id: TypeId) -> Option<usize> {
        unsafe {
            let slice = &*std::ptr::slice_from_raw_parts(self.ids, self.component_count);
            match slice.binary_search(&type_id) {
                Ok(idx) => Some(idx),
                Err(_) => None,
            }
        }
    }

    #[inline(always)]
    pub fn component_count(&self) -> usize {
        self.component_count
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.component_count == 0
    }

    #[inline]
    pub(crate) unsafe fn component_sizes(&self) -> *const usize {
        self.sizes
    }

    pub fn is_same_as(&self, other: &Self) -> bool {
        if self.component_count != other.component_count {
            return false;
        }

        if self.is_empty() {
            return true;
        }

        let ids = unsafe { std::slice::from_raw_parts(self.ids, self.component_count) };
        let other_ids =
            unsafe { std::slice::from_raw_parts(other.ids, other.component_count) };

        return ids == other_ids;
    }

    #[inline(always)]
    pub fn is_include(&self, other: &Self) -> bool {
        self.is_include_ids(other.included_ids())
    }

    pub fn is_include_ids(&self, ids_to_check: &[TypeId]) -> bool {
        let self_ids = self.included_ids();

        if self_ids.len() < ids_to_check.len() {
            return false;
        }

        for id in ids_to_check {
            let result = self_ids.binary_search(id);
            if result.is_err() {
                return false;
            }
        }

        return true;
    }

    #[inline(always)]
    pub fn is_exclude(&self, other: &Self) -> bool {
        self.is_exclude_ids(other.included_ids())
    }

    pub fn is_exclude_ids(&self, ids_to_check: &[TypeId]) -> bool {
        let self_ids = self.included_ids();

        let mut left_idx = 0;
        for id in ids_to_check {
            let result = self_ids[left_idx..].binary_search(id);
            match result {
                Ok(_) => return false,
                Err(found_idx) => {
                    if found_idx >= self_ids.len() {
                        break;
                    } else {
                        left_idx = found_idx;
                    }
                }
            }
        }

        return true;
    }

    #[inline(always)]
    pub fn included_ids(&self) -> &[TypeId] {
        unsafe { &*std::ptr::slice_from_raw_parts(self.ids, self.component_count) }
    }
}

impl Clone for Archetype {
    fn clone(&self) -> Self {
        let component_count = self.component_count;
        let (ids, sizes, aligns) = unsafe {
            (
                mem_utils::alloc(component_count),
                mem_utils::alloc(component_count),
                mem_utils::alloc(component_count),
            )
        };

        unsafe {
            self.ids.copy_to_nonoverlapping(ids, component_count);
            self.sizes.copy_to_nonoverlapping(sizes, component_count);
            self.aligns.copy_to_nonoverlapping(aligns, component_count);
        }

        Self {
            ids,
            sizes,
            aligns,
            component_count,
        }
    }
}

impl Hash for Archetype {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in 0..self.component_count {
            let id = unsafe { *self.ids.add(i) };

            id.hash(state);
        }
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        unsafe {
            mem_utils::dealloc(self.ids, self.component_count);
            mem_utils::dealloc(self.sizes, self.component_count);
            mem_utils::dealloc(self.aligns, self.component_count);
        }
    }
}

macro_rules! tuple_into_archetype_impl {
    ($($T: tt),*) => {
        #[allow(unused_parens)]
        impl<$($T: 'static + Component),*> $crate::archetype::ComponentTupleToArchetype for ($($T),*) {
            fn into_archetype() -> $crate::Archetype
            {
                let components = &mut $crate::component_types!( $($T),* );
                components.sort_by_key(|a| a.id());
                $crate::Archetype::new(components)
            }
        }
    };
}

pub(crate) use tuple_into_archetype_impl;
