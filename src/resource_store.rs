use std::{ops::{Index, IndexMut}, mem::ManuallyDrop};

use crate::{entities_container::EntitiesContainer, mem_utils, Entity};

pub struct ResourceStore<T> {
    elements: *mut ManuallyDrop<T>,
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
        let old_capacity = self.entities_container.capacity();
        let creation = self.entities_container.create_entity();
        let entity = creation.entity;

        if creation.container_was_grow() {
            self.elements = unsafe {
                mem_utils::realloc(
                    self.elements,
                    old_capacity,
                    self.entities_container.capacity(),
                )
            };
        }

        unsafe {
            self.elements.add(entity.id as usize).write(ManuallyDrop::new(resource));
        }

        entity
    }

    pub fn release(&mut self, entity: Entity) -> T {
        debug_assert!(self.entities_container.is_alive(entity));
        self.entities_container.destroy_entity(entity);
        unsafe { 
            ManuallyDrop::take(&mut self.elements.add(entity.id as usize).read())
        }
    }
}

impl<T> Index<Entity> for ResourceStore<T> {
    type Output = T;

    fn index(&self, entity: Entity) -> &Self::Output {
        debug_assert!(self.entities_container.is_alive(entity));
        unsafe {
            self.elements
                .add(entity.id as usize)
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

impl<T> IndexMut<Entity> for ResourceStore<T> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        debug_assert!(self.entities_container.is_alive(entity));
        unsafe {
            self.elements
                .add(entity.id as usize)
                .as_mut()
                .unwrap_unchecked()
        }
    }
}

impl<T> Drop for ResourceStore<T> {
    fn drop(&mut self) {
        for i in 0..self.entities_container.capacity() {
            if !self.entities_container.is_alive_at_index(i) {
                continue;
            }

            unsafe { ManuallyDrop::drop(&mut self.elements.add(i).read()) };
        }
    }
}
