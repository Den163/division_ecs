use crate::{entity_in_archetype::EntityInArchetype, mem_utils, Entity};

#[derive(Debug)]
pub(crate) struct EntitiesContainer {
    gap_ids: Vec<u32>,
    capacity: usize,
    entity_to_version: *mut u32,
    entity_to_archetype: *mut EntityInArchetype,
    entity_to_is_alive: *mut u32,
    next_free_id: u32,
}

impl EntitiesContainer {
    pub fn new(capacity: usize) -> EntitiesContainer {
        let entity_to_version = mem_utils::alloc_zeroed(capacity);
        let entity_to_archetype = mem_utils::alloc_zeroed(capacity);
        let entity_to_is_alive = mem_utils::alloc_zeroed(get_bit_vec_size_for_capacity(capacity));
        let gap_ids = Vec::new();

        EntitiesContainer {
            capacity,
            entity_to_version,
            entity_to_archetype,
            entity_to_is_alive,
            gap_ids,
            next_free_id: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn grow(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }

        let old_capacity = self.capacity;
        let alive_old_capacity = get_bit_vec_size_for_capacity(old_capacity);
        let alive_new_capacity = get_bit_vec_size_for_capacity(new_capacity);

        self.entity_to_version = mem_utils::realloc_with_uninit_capacity_zeroing(
            self.entity_to_version,
            old_capacity,
            new_capacity,
        );
        self.entity_to_archetype = mem_utils::realloc_with_uninit_capacity_zeroing(
            self.entity_to_archetype,
            old_capacity,
            new_capacity,
        );

        if alive_old_capacity != alive_new_capacity {
            self.entity_to_is_alive = mem_utils::realloc_with_uninit_capacity_zeroing(
                self.entity_to_is_alive,
                alive_old_capacity,
                alive_new_capacity,
            );
        }

        self.capacity = new_capacity;
    }

    pub fn create_entity(&mut self) -> Entity {
        let gap_count = self.gap_ids.len();
        let id = if gap_count > 0 {
            self.gap_ids.remove(gap_count - 1)
        } else {
            let id = self.next_free_id;
            self.next_free_id += 1;

            if id >= self.capacity as u32 {
                self.grow(self.capacity * 2);
            }

            id
        };

        let usid = id as usize;
        let version;
        unsafe {
            self.toggle_alive(id);
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
        self.toggle_alive(id);

        if id == self.next_free_id - 1 {
            self.next_free_id = id;
        } else {
            self.gap_ids.push(id);
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.validate_id_with_panic(entity.id);

        let mask_index = entity.id / 32;
        let mask_bit = entity.id % 32;
        let alive_mask = 1 >> mask_bit;

        unsafe {
            self.validate_entity_version(entity)
                & ((*self.entity_to_is_alive.add(mask_index as usize) & alive_mask) == alive_mask)
        }
    }

    #[inline(always)]
    pub fn get_entity_in_archetype(&self, id: u32) -> EntityInArchetype {
        self.validate_id_with_panic(id);
        unsafe { *self.entity_to_archetype.add(id as usize) }
    }

    #[inline(always)]
    pub(crate) fn get_entity_by_id(&self, entity_id: u32) -> Entity {
        Entity {
            version: unsafe { *self.entity_to_version.add(entity_id as usize) },
            id: entity_id
        }
    }

    #[inline(always)]
    pub(crate) fn get_entity_versions(&self) -> &[u32] {
        unsafe {
            &*std::ptr::slice_from_raw_parts(self.entity_to_version, self.capacity)
        }
    }

    #[inline(always)]
    pub fn set_entity_in_archetype(&self, id: u32, entity_in_archetype: EntityInArchetype) {
        self.validate_id_with_panic(id);
        unsafe {
            *self.entity_to_archetype.add(id as usize) = entity_in_archetype;
        }
    }

    fn toggle_alive(&self, id: u32) {
        let mask_index = id / 32;
        let mask_bit = id as u32 % 32;
        let alive_mask = 1 >> mask_bit;

        unsafe {
            let mask_ptr = self.entity_to_is_alive.add(mask_index as usize);
            *mask_ptr ^= alive_mask;
        }
    }

    #[inline(always)]
    fn validate_id(&self, id: u32) -> bool {
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

    fn validate_id_with_panic(&self, id: u32) {
        assert!(
            self.validate_id(id),
            "Invalid entity id (Maybe it's from another world)"
        );
    }
}

impl Drop for EntitiesContainer {
    fn drop(&mut self) {
        mem_utils::dealloc(
            self.entity_to_is_alive,
            get_bit_vec_size_for_capacity(self.capacity),
        );
        mem_utils::dealloc(self.entity_to_version, self.capacity);
        mem_utils::dealloc(self.entity_to_archetype, self.capacity);
    }
}

fn get_bit_vec_size_for_capacity(capacity: usize) -> usize {
    capacity / 32 + 1
}
