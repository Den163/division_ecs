use std::ops::{Index, IndexMut};

use crate::{entities_container::EntitiesContainer, mem_utils, Entity};

pub struct ResourceStore<T> {
    elements: *mut T,
    entities_container: EntitiesContainer,
}

impl<T> ResourceStore<T> {
    pub fn new() -> ResourceStore<T> {
        ResourceStore {
            elements: std::ptr::null_mut(),
            entities_container: EntitiesContainer::new(0),
        }
    }

    pub fn with_capacity(capacity: usize) -> ResourceStore<T> {
        ResourceStore {
            elements: unsafe { mem_utils::alloc(capacity) },
            entities_container: EntitiesContainer::new(capacity),
        }
    }

    pub fn create(&mut self, resource: T) -> Entity {
        let will_grow = self.entities_container.will_grow_with_entity_create();
        let old_capacity = self.entities_container.capacity();
        let e = self.entities_container.create_entity();

        if will_grow {
            self.elements = unsafe {
                mem_utils::realloc(
                    self.elements,
                    old_capacity,
                    self.entities_container.capacity(),
                )
            };
        }

        unsafe {
            std::ptr::write(
                self.elements.add(e.id as usize),
                resource,
            );
        }

        e
    }

    pub fn release(&mut self, entity: Entity) -> T {
        debug_assert!(self.entities_container.is_alive(entity));
        self.entities_container.destroy_entity(entity);
        unsafe {
            std::ptr::read(self.elements.add(entity.id as usize))
        }
    }
}

impl<T> Index<Entity> for ResourceStore<T> {
    type Output = T;

    fn index(&self, entity: Entity) -> &Self::Output {
        debug_assert!(self.entities_container.is_alive(entity));
        unsafe { &*self.elements.add(entity.id as usize) as &T }
    }
}

impl<T> IndexMut<Entity> for ResourceStore<T> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        debug_assert!(self.entities_container.is_alive(entity));
        unsafe { &mut *self.elements.add(entity.id as usize) as &mut T }
    }
}

impl<T> Drop for ResourceStore<T> {
    fn drop(&mut self) {
        for i in 0..self.entities_container.capacity() {
            if !self.entities_container.is_alive_at_index(i) {
                continue;
            }

            unsafe { self.elements.add(i).drop_in_place() };
        }
    }
}
