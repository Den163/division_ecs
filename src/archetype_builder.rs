use crate::{component_type::ComponentType, archetype::Archetype};

pub struct ArchetypeBuilder {
    component_types: Vec<ComponentType>,
}

impl ArchetypeBuilder {
    pub fn new() -> ArchetypeBuilder {
        ArchetypeBuilder { component_types: Vec::new() }
    }

    pub fn component<T: 'static>(mut self) -> Self {
        self.component_types.push(ComponentType::of::<T>());
        self
    }

    pub fn build(self) -> Archetype {
        Archetype::new(&self.component_types)
    }
}