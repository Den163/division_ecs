use crate::{Entity, EntityRequestError, InternalEntity};

#[derive(Debug)]
pub struct EntitiesContainer  {
    sparse_: Vec<InternalEntity>,
    dense_: Vec<i32>,
    free_list_: Vec<i32>
}

impl EntitiesContainer {
    pub fn new(capacity: usize) -> EntitiesContainer {
        return EntitiesContainer {
            sparse_: (0..capacity).map(|_| InternalEntity { alive: false, version: 0 }).collect(),
            dense_: Vec::with_capacity(capacity),
            free_list_: Vec::with_capacity(capacity),
        };
    }

    pub fn create_entity(&mut self) -> Entity {
        todo!();
    }

    pub fn try_destroy_entity(&mut self, _entity: Entity) -> Result<(), EntityRequestError> {
        todo!();
    }

    pub fn destroy_entity(&mut self, _entity: Entity) {
        todo!();
    }

    pub fn is_alive(&self, _entity: Entity) -> bool {
        todo!();
    }
}