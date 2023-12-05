use std::{
    any::TypeId,
    hash::{Hash, Hasher},
};

use crate::{
    archetype_data_page::ArchetypeDataPage, component_type::ComponentType, mem_utils,
    component_tuple::ComponentTuple,
};

#[derive(Debug)]
pub struct Archetype {
    ids: *mut TypeId,
    sizes: *mut usize,
    aligns: *mut usize,
    offsets: *mut usize,
    component_count: usize,
    entities_capacity: usize,
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

        let entities_capacity =
            unsafe { Self::calculate_entities_capacity(sizes, aligns, component_count) };
        let offsets = unsafe {
            Self::calculate_offsets(sizes, aligns, component_count, entities_capacity)
        };

        Archetype {
            ids,
            sizes,
            aligns,
            offsets,
            component_count,
            entities_capacity,
        }
    }

    unsafe fn calculate_entities_capacity(
        sizes: *const usize,
        aligns: *const usize,
        component_count: usize,
    ) -> usize {
        let ptr_size = std::mem::size_of::<usize>();

        let aligns = std::slice::from_raw_parts(aligns, component_count);
        let max_align = aligns.into_iter().max().unwrap();
        let max_align = max_align % ptr_size;

        let bytes_per_components_row_approx =
            ArchetypeDataPage::PAGE_SIZE_BYTES / component_count - max_align;

        let sizes = std::slice::from_raw_parts(sizes, component_count);
        let entities_capacity = sizes
            .iter()
            .map(|s| bytes_per_components_row_approx / *s)
            .min()
            .unwrap();

        entities_capacity
    }

    unsafe fn calculate_offsets(
        sizes: *const usize,
        aligns: *const usize,
        component_count: usize,
        entities_capacity: usize,
    ) -> *mut usize {
        let component_offsets: *mut usize = mem_utils::alloc(component_count);
        let mut offset = 0;

        for i in 0..component_count {
            let size = *sizes.add(i);
            let align = *aligns.add(i);
            let align_offset = offset % align;

            if align_offset != 0 {
                offset += align - align_offset;
            }

            *component_offsets.add(i) = offset;

            offset += size * entities_capacity;
        }

        assert!(offset <= ArchetypeDataPage::PAGE_SIZE_BYTES);
        component_offsets
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

    #[inline(always)]
    pub fn component_count(&self) -> usize {
        self.component_count
    }

    #[inline(always)]
    pub fn entities_capacity(&self) -> usize {
        self.entities_capacity
    }

    #[inline]
    pub(crate) unsafe fn component_sizes(&self) -> *const usize {
        self.sizes
    }

    #[inline]
    pub(crate) unsafe fn component_offsets(&self) -> *const usize {
        self.offsets
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
        let (ids, sizes, aligns, offsets) = unsafe {
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
            self.offsets
                .copy_to_nonoverlapping(offsets, component_count);
        }

        Self {
            ids,
            sizes,
            aligns,
            offsets,
            component_count,
            entities_capacity: self.entities_capacity,
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
            mem_utils::dealloc(self.offsets, self.component_count);
        }
    }
}