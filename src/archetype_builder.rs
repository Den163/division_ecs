use crate::{
    archetype::Archetype, component_type::ComponentType, component_tuple::ComponentTuple,
};

pub struct ArchetypeBuilder {
    component_types: Vec<ComponentType>,
}

impl ArchetypeBuilder {
    pub fn new() -> ArchetypeBuilder {
        ArchetypeBuilder {
            component_types: Vec::new(),
        }
    }

    pub fn include_components<T: ComponentTuple>(&mut self) -> &mut Self {
        T::add_components_to_archetype_builder(self)
    }

    pub fn exclude_components<T: ComponentTuple>(&mut self) -> &mut Self {
        T::remove_components_from_archetype_builder(self)
    }

    pub fn include_component_types(&mut self, components: &[ComponentType]) -> &mut Self {
        for comp in components {
            if !self.component_types.contains(comp) {
                self.component_types.push(*comp);
            }
        }

        self
    }

    pub fn exclude_component_types(&mut self, components: &[ComponentType]) -> &mut Self {
        for comp in components {
            if let Some(idx) = self.component_types.iter().position(|c| c == comp) {
                self.component_types.remove(idx);
            }
        }

        self
    }

    pub fn include_archetype(&mut self, archetype: &Archetype) -> &mut Self {
        self.component_types
            .reserve(self.component_types.capacity() + archetype.component_count());

        for comp in archetype.components_iter() {
            if !self.component_types.contains(&comp) {
                self.component_types.push(comp)
            }
        }

        self
    }

    pub fn build(&mut self) -> Archetype {
        self.component_types.sort_by_key(|a| a.id());

        Archetype::new(&mut self.component_types)
    }
}