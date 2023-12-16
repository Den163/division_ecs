use std::{
    any::TypeId,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use crate::{component_tuple::ComponentTuple, component_type::ComponentType, mem_utils};

#[derive(Debug)]
pub struct Archetype {
    ids: *mut TypeId,
    sizes: *mut usize,
    aligns: *mut usize,
    names: *mut &'static str,
    component_count: usize,
}

pub struct ArchetypesUnion {
    pub lhs_indices: Vec<usize>,
    pub rhs_indices: Vec<usize>,
}

impl Archetype {
    pub fn with_components<T: ComponentTuple>() -> Self {
        T::into_archetype()
    }

    pub(crate) fn new(sorted_components: &[ComponentType]) -> Self {
        let component_count = sorted_components.len();
        assert!(component_count > 0);

        let (ids, sizes, aligns, names): (
            *mut TypeId,
            *mut usize,
            *mut usize,
            *mut &'static str,
        ) = unsafe {
            (
                mem_utils::alloc(component_count),
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
                *names.add(i) = comp.name();
            };
        }

        Archetype {
            ids,
            sizes,
            aligns,
            names,
            component_count,
        }
    }

    #[inline(always)]
    pub fn has_component<T: 'static>(&self) -> bool {
        self.find_component_index(TypeId::of::<T>()).is_some()
    }

    #[inline(always)]
    pub fn components_iter<'a>(&'a self) -> impl Iterator<Item = ComponentType> + 'a {
        (0..self.component_count).into_iter().map(|i| unsafe {
            ComponentType::new(
                *self.ids.add(i),
                *self.sizes.add(i),
                *self.aligns.add(i),
                *self.names.add(i),
            )
        })
    }

    pub fn find_component_index_of<T: 'static>(&self) -> Option<usize> {
        self.find_component_index(std::any::TypeId::of::<T>())
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

    #[inline]
    pub fn component_count(&self) -> usize {
        self.component_count
    }

    #[inline]
    pub(crate) fn component_aligns(&self) -> *const usize {
        self.aligns
    }

    #[inline]
    pub(crate) fn component_sizes(&self) -> *const usize {
        self.sizes
    }

    pub fn is_same_as(&self, other: &Self) -> bool {
        if self.component_count != other.component_count {
            return false;
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

    pub fn is_include_only_ids(&self, ids_to_check: &[TypeId]) -> bool {
        let self_ids = self.included_ids();

        if self_ids.len() != ids_to_check.len() {
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

impl ArchetypesUnion {
    pub fn calculate(lhs: &Archetype, rhs: &Archetype) -> Self {
        let mut lhs_indices = Vec::new();
        let mut rhs_indices = Vec::new();

        let (outer, inner, outer_indices, inner_indices) =
            if lhs.component_count > rhs.component_count {
                (lhs, rhs, &mut lhs_indices, &mut rhs_indices)
            } else {
                (rhs, lhs, &mut rhs_indices, &mut lhs_indices)
            };

        let mut o = 0;

        while o < outer.component_count {
            let outer_type_id = unsafe { *outer.ids.add(o) };

            let mut i = 0;
            while i < inner.component_count {
                let inner_type_id = unsafe { *inner.ids.add(i) };

                if inner_type_id > outer_type_id {
                    break;
                }

                if inner_type_id == outer_type_id {
                    inner_indices.push(i);
                    outer_indices.push(o);
                }

                i += 1;
            }

            o += 1;
        }

        ArchetypesUnion {
            lhs_indices,
            rhs_indices,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.lhs_indices.len()
    }
}

impl Clone for Archetype {
    fn clone(&self) -> Self {
        let component_count = self.component_count;
        let (ids, sizes, aligns, names) = unsafe {
            (
                mem_utils::alloc(component_count),
                mem_utils::alloc(component_count),
                mem_utils::alloc(component_count),
                mem_utils::alloc(component_count),
            )
        };

        unsafe {
            self.ids.copy_to_nonoverlapping(ids, component_count);
            self.sizes.copy_to_nonoverlapping(sizes, component_count);
            self.aligns.copy_to_nonoverlapping(aligns, component_count);
            self.names.copy_to_nonoverlapping(names, component_count)
        }

        Self {
            ids,
            sizes,
            aligns,
            names,
            component_count,
        }
    }
}

impl Display for Archetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archetype")
            .field("Components:", unsafe {
                &std::slice::from_raw_parts(self.names, self.component_count)
            })
            .finish()
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
            mem_utils::dealloc(self.names, self.component_count);
        }
    }
}
