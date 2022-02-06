use crate::{EntitiesContainer, Entity};
use crate::errors::EntityRequestError;

const ENTITIES_DEFAULT_CAPACITY: usize = 10;

#[derive(Debug)]
pub struct Registry {
    _entities_container: EntitiesContainer
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            _entities_container: EntitiesContainer::new(ENTITIES_DEFAULT_CAPACITY)
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self._entities_container.create_entity()
    }

    pub fn try_destroy_entity(&mut self, entity: Entity) -> Result<(), EntityRequestError> {
        self._entities_container.try_destroy_entity(entity)
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self._entities_container.destroy_entity(entity)
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self._entities_container.is_alive(entity)
    }
}

