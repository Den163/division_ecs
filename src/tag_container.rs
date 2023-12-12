use std::any::TypeId;

use crate::{bitvec_utils, derived_traits::Tag, Store, Entity};

pub(crate) struct TagContainer {
    tag_ids: Vec<TypeId>,
    entity_id_to_has_tag_bitvecs: Vec<*mut u32>,
    entity_capacity: usize,
}

impl TagContainer {
    pub fn new(entity_capacity: usize) -> Self {
        Self {
            tag_ids: Vec::new(),
            entity_id_to_has_tag_bitvecs: Vec::new(),
            entity_capacity,
        }
    }

    pub fn has_tag_bitvec<T: Tag + 'static>(&self) -> *const u32 {
        let type_id = TypeId::of::<T>();
        let tag_index = match self.tag_ids.binary_search(&type_id) {
            Ok(i) => i,
            Err(_) => panic!("Failed to find tag with id: {type_id:?}")
        };

        unsafe {
            *self.entity_id_to_has_tag_bitvecs.get_unchecked(tag_index)
        }
    }

    pub fn add_tag<T: Tag + 'static>(&mut self, entity_id: u32) {
        let type_id = TypeId::of::<T>();

        let tag_index = match self.tag_ids.binary_search(&type_id) {
            Ok(i) => i,
            Err(i) => {
                let has_tag_bitvec = unsafe { bitvec_utils::alloc(self.entity_capacity) };
                self.tag_ids.insert(i, type_id);
                self.entity_id_to_has_tag_bitvecs.insert(i, has_tag_bitvec);

                i
            }
        };

        unsafe {
            let bitvec = *self
                .entity_id_to_has_tag_bitvecs
                .get_unchecked_mut(tag_index);
            bitvec_utils::set_bit_on(bitvec, entity_id as usize);
        };
    }

    pub fn remove_tag<T: Tag + 'static>(&mut self, entity_id: u32) {
        let type_id = TypeId::of::<T>();
        match self.tag_ids.binary_search(&type_id) {
            Ok(i) => unsafe {
                let bitvec = *self.entity_id_to_has_tag_bitvecs.get_unchecked_mut(i);
                bitvec_utils::set_bit_off(bitvec, entity_id as usize);
            },
            Err(_) => {}
        };
    }

    pub fn has_tag<T: Tag + 'static>(&self, entity_id: u32) -> bool {
        let type_id = TypeId::of::<T>();
        match self.tag_ids.binary_search(&type_id) {
            Ok(i) => unsafe {
                let bitvec = *self.entity_id_to_has_tag_bitvecs.get_unchecked(i);
                bitvec_utils::is_bit_on(bitvec, entity_id as usize)
            },
            Err(_) => false,
        }
    }

    pub fn remove_all_tags_for_entity(&mut self, entity_id: u32) {
        for &mut bitvec in &mut self.entity_id_to_has_tag_bitvecs {
            unsafe {
                bitvec_utils::set_bit_off(bitvec, entity_id as usize);
            }
        }
    }

    pub fn grow(&mut self, new_capacity: usize) {
        for bitvec in &mut self.entity_id_to_has_tag_bitvecs {
            unsafe {
                *bitvec = 
                    bitvec_utils::realloc(*bitvec, self.entity_capacity, new_capacity);
            }
        }

        self.entity_capacity = new_capacity;
    }
}

impl Store {
    #[inline(always)]
    pub fn add_tag<T: Tag + 'static>(&mut self, entity: Entity) {
        self.tag_container.add_tag::<T>(entity.id);
    }

    #[inline(always)]
    pub fn remove_tag<T: Tag + 'static>(&mut self, entity: Entity) {
        self.tag_container.remove_tag::<T>(entity.id);
    }

    #[inline(always)]
    pub fn has_tag<T: Tag + 'static>(&self, entity: Entity) -> bool {
        self.tag_container.has_tag::<T>(entity.id)
    }
}

impl Drop for TagContainer {
    fn drop(&mut self) {
        for &mut bitvec in &mut self.entity_id_to_has_tag_bitvecs {
            unsafe {
                bitvec_utils::dealloc(bitvec, self.entity_capacity);
            }
        }
    }
}
