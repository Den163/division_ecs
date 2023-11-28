use crate::{bitvec_utils, mem_utils, Entity};

#[derive(Debug)]
pub(crate) struct EntitiesContainer {
    gap_ids: Vec<u32>,
    capacity: usize,
    entity_to_version: *mut u32,
    entity_to_is_alive_bitvec: *mut u32,
    next_free_id: u32,
}

impl EntitiesContainer {
    pub fn new(capacity: usize) -> EntitiesContainer {
        let (entity_to_version, entity_to_is_alive) = unsafe {
            (
                mem_utils::alloc_zeroed(capacity),
                bitvec_utils::alloc(capacity),
            )
        };

        let gap_ids = Vec::new();

        EntitiesContainer {
            capacity,
            entity_to_version,
            entity_to_is_alive_bitvec: entity_to_is_alive,
            gap_ids,
            next_free_id: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn will_grow_with_entity_create(&self) -> bool {
        (self.gap_ids.len() == 0) & (self.next_free_id >= self.capacity as u32)
    }

    pub fn will_grow_with_id(&self, id: u32) -> bool {
        id >= self.capacity as u32
    }

    pub fn grow(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }

        let old_capacity = self.capacity;

        unsafe {
            self.entity_to_version = mem_utils::realloc_with_uninit_capacity_zeroing(
                self.entity_to_version,
                old_capacity,
                new_capacity,
            );

            self.entity_to_is_alive_bitvec = bitvec_utils::realloc(
                self.entity_to_is_alive_bitvec,
                old_capacity,
                new_capacity,
            );
        };

        self.capacity = new_capacity;
    }

    pub fn create_entity(&mut self) -> Entity {
        let gap_count = self.gap_ids.len();
        let id = if gap_count > 0 {
            self.gap_ids.remove(gap_count - 1)
        } else {
            let id = self.next_free_id;
            self.next_free_id += 1;

            if self.will_grow_with_id(id) {
                self.grow(std::cmp::max(self.capacity, 1) * 2);
            }

            id
        };

        let usid = id as usize;
        let version;
        unsafe {
            bitvec_utils::toggle_bit(self.entity_to_is_alive_bitvec, usid);
            let version_ptr = self.entity_to_version.add(usid);

            version = *version_ptr + 1;
            *version_ptr = version;
        }

        Entity { id, version }
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.validate_entity_version_with_panic(entity);

        let id = entity.id;
        assert!(self.is_alive(entity), "Entity is already dead");
        unsafe {
            bitvec_utils::toggle_bit(self.entity_to_is_alive_bitvec, id as usize);
        }

        if id == self.next_free_id - 1 {
            self.next_free_id = id;
        } else {
            self.gap_ids.push(id);
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.validate_id_with_panic(entity.id);

        unsafe {
            self.validate_entity_version(entity)
                & bitvec_utils::is_bit_on(
                    self.entity_to_is_alive_bitvec,
                    entity.id as usize,
                )
        }
    }

    #[inline(always)]
    pub(crate) fn is_alive_at_index(&self, index: usize) -> bool {
        unsafe { bitvec_utils::is_bit_on(self.entity_to_is_alive_bitvec, index) }
    }

    #[inline(always)]
    pub(crate) fn get_entity_versions(&self) -> &[u32] {
        unsafe { &*std::ptr::slice_from_raw_parts(self.entity_to_version, self.capacity) }
    }

    #[inline(always)]
    pub fn validate_id(&self, id: u32) -> bool {
        (id as usize) < self.capacity
    }

    #[inline(always)]
    fn validate_entity_version(&self, entity: Entity) -> bool {
        unsafe { *self.entity_to_version.add(entity.id as usize) == entity.version }
    }

    fn validate_entity_version_with_panic(&self, entity: Entity) {
        self.validate_id_with_panic(entity.id);
        assert!(
            self.validate_entity_version(entity),
            "Invalid entity version (It's dead)"
        );
    }

    pub fn validate_id_with_panic(&self, id: u32) {
        assert!(
            self.validate_id(id),
            "Invalid entity id (Maybe it's from another world)"
        );
    }
}

impl Drop for EntitiesContainer {
    fn drop(&mut self) {
        unsafe {
            bitvec_utils::dealloc(self.entity_to_is_alive_bitvec, self.capacity);
            mem_utils::dealloc(self.entity_to_version, self.capacity);
        }
    }
}
