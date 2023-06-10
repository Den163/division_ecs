use crate::{EntitiesContainer, Entity, archetype::Archetype};


const ENTITIES_DEFAULT_CAPACITY: usize = 10;

#[derive(Debug)]
pub struct Registry {
    entities_container: EntitiesContainer,
    archetypes: Vec<Archetype>,
}

impl Registry {
    pub fn new() -> Registry {
        Self::with_capacity(ENTITIES_DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Registry {
        Registry { 
            entities_container: EntitiesContainer::new(capacity),
            archetypes: Vec::new()
        }
    }

    #[inline(always)]
    pub fn entities_capacity(&self) -> usize {
        self.entities_container.capacity()
    }

    #[inline(always)]
    pub fn create_entity(&mut self) -> Entity {
        self.entities_container.create_entity()
    }

    #[inline(always)]
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entities_container.destroy_entity(entity)
    }

    #[inline(always)]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities_container.is_alive(entity)
    }
}

