use std::collections::HashMap;
use crate::Entity;
use crate::errors::EntityRequestError;

#[derive(Debug, Clone, Copy, PartialEq)]
struct InternalEntity {
    _exist: bool,
    _version: u32
}

impl Default for InternalEntity {
    fn default() -> Self { return InternalEntity { _exist: false, _version: 0 }; }
}

#[derive(Debug)]
pub struct Registry {
    _component_id_to_ptr: HashMap<usize, usize>,
    _entities: Vec<InternalEntity>
}

impl Registry {
    pub fn new() -> Registry {
        return Registry {
            _component_id_to_ptr : HashMap::new(),
            _entities: vec![InternalEntity::default(); 1]
        };
    }

    pub fn create_entity(&mut self) -> Entity {
        let entities = &mut self._entities;
        for (index, e) in entities.iter_mut().enumerate() {
            if e._exist { continue; }

            return Self::prepare_and_get_entity(e, index);
        }

        let entities_count_before = self._entities.len();
        self.resize_entities(self._entities.len() * 2);

        return Self::prepare_and_get_entity(
            &mut self._entities[entities_count_before],
            entities_count_before
        );
    }

    pub fn try_destroy_entity(&mut self, entity: Entity) -> Result<(), EntityRequestError> {
        let id = entity.id as usize;
        if id >= self._entities.len() { return Err(EntityRequestError::InvalidId); }

        let e = self._entities[id];
        if !e._exist || entity.version != e._version {
            return Err(EntityRequestError::DeadEntity);
        }

        self.destroy_entity(entity);

        return Ok(());
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self._entities[entity.id as usize]._exist = false;
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let id = entity.id as usize;
        return id < self._entities.len() &&
               self._entities[id]._exist &&
               self._entities[id]._version == entity.version;
    }

    fn prepare_and_get_entity(internal_entity: &mut InternalEntity, index: usize) -> Entity {
        internal_entity._exist = true;
        internal_entity._version += 1;

        return Entity {
            id : index as u32,
            version : internal_entity._version
        };
    }

    fn resize_entities(&mut self, new_size: usize) -> () {
        debug_assert!(new_size > self._entities.len());

        self._entities.resize(
            new_size,
            InternalEntity::default()
        );
    }
}

