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
        for (i, e) in self.sparse_.iter_mut().enumerate() {
            if !e.alive {
                e.version += 1;
                e.alive = true;

                return  Entity {
                    id: i as u32,
                    version: e.version
                };
            }
        }

        panic!("Not implemented");
    }

    pub fn try_destroy_entity(&mut self, _entity: Entity) -> Result<(), EntityRequestError> {
        panic!("Not implemented");
    }

    pub fn destroy_entity(&mut self, _entity: Entity) {
        panic!("Not implemented");
    }

    pub fn is_alive(&self, _entity: Entity) -> bool {
        panic!("Not implemented");
    }
}