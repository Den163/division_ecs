use crate::{Entity, mem_utils};

#[derive(Debug)]
pub struct EntitiesContainer  {
    capacity: usize,
    entity_to_version: *mut u32,
    entity_to_is_alive: *mut bool,
    gap_ids: Vec<u32>,
    next_free_id: u32
}

impl EntitiesContainer {
    pub fn new(capacity: usize) -> EntitiesContainer {
        let entity_to_version = mem_utils::alloc_zeroed(capacity);
        let entity_to_is_alive = mem_utils::alloc_zeroed(capacity);
        let gap_ids = Vec::new();

        EntitiesContainer { capacity, entity_to_version, entity_to_is_alive, gap_ids, next_free_id: 0 }
    }

    pub fn capacity(&self) -> usize { 
        self.capacity
    }

    pub fn grow(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity { 
            return;
        }

        let entity_to_version = mem_utils::alloc_zeroed(new_capacity);
        let entity_to_is_alive = mem_utils::alloc_zeroed(new_capacity);

        unsafe {
            self.entity_to_version.copy_to(entity_to_version, self.capacity);
            self.entity_to_is_alive.copy_to(entity_to_is_alive, self.capacity);
        }
        
        mem_utils::dealloc(self.entity_to_version, self.capacity);
        mem_utils::dealloc(self.entity_to_is_alive, self.capacity);

        self.entity_to_version = entity_to_version;
        self.entity_to_is_alive = entity_to_is_alive;
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
        let version ;
        unsafe { 
            *self.entity_to_is_alive.add(usid) = true;
            let version_ptr = self.entity_to_version.add(usid);

            version = *version_ptr + 1;
            *version_ptr = version;
        }

        Entity { id, version }
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.validate_entity_version_with_panic(entity);

        let id = entity.id;
        unsafe {
            let alive_ptr = self.entity_to_is_alive.add(id as usize);
            debug_assert!(*alive_ptr == true, "Entity is already dead");

            *alive_ptr = false;
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
            self.validate_entity_version(entity) &
            *self.entity_to_is_alive.add(entity.id as usize)
        }
    }

    #[inline(always)]
    fn validate_id(&self, id: u32) -> bool {
        (id as usize) < self.capacity
    }

    #[inline(always)]
    fn validate_entity_version(&self, entity: Entity) -> bool {
        unsafe {
            *self.entity_to_version.add(entity.id as usize) == entity.version
        }
    }

    fn validate_entity_version_with_panic(&self, entity: Entity) {
        self.validate_id_with_panic(entity.id);
        debug_assert!(self.validate_entity_version(entity), "Invalid entity version (It's dead)");
    }


    fn validate_id_with_panic(&self, id: u32) {
        debug_assert!(self.validate_id(id), "Invalid entity id (Maybe it's from another world)");
    }
}

impl Drop for EntitiesContainer {
    fn drop(&mut self) {
        mem_utils::dealloc(self.entity_to_is_alive, self.capacity);
        mem_utils::dealloc(self.entity_to_version, self.capacity);
    }
}