use std::{any::TypeId};

use crate::{component_type::{ComponentType}};

#[derive(Debug)]
pub struct Archetype {
    component_types: Vec<ComponentType>
}

impl Archetype {
    pub(crate) fn new(components: &[ComponentType]) -> Self {
        debug_assert!(components.len() > 0);

        let mut component_types = components.to_vec();
        component_types.sort_by_key(|c| { c.id() });
        
        Archetype { component_types }
    }

    pub fn has_component<T: 'static>(&self) -> bool {
        self.find_component_index(TypeId::of::<T>()).is_some()
    }

    pub fn components(&self) -> &[ComponentType] {
        &self.component_types
    }

    pub(crate) fn find_component_index(&self, type_id: TypeId) -> Option<usize> {
        for i in 0..self.component_types.len() {
            let comp = &self.component_types[i];
            if comp.id() == type_id {
                return Some(i);
            }
        }

        return None;
    }
}