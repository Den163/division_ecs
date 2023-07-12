use crate::{component_type::ComponentType, archetype::Archetype};

pub struct ArchetypeBuilder {
    component_types: Vec<ComponentType>,
}

impl ArchetypeBuilder {
    pub fn new() -> ArchetypeBuilder {
        ArchetypeBuilder { component_types: Vec::new() }
    }

    pub fn component<T: 'static>(mut self) -> Self {
        self = self.component_type(ComponentType::of::<T>());

        self
    }

    pub fn component_types(mut self, components: &[ComponentType]) -> Self {
        for comp in components {
            self = self.component_type(*comp)
        }

        self
    }

    pub fn component_type(mut self, component: ComponentType) -> Self {
        if !self.component_types.contains(&component) {
            self.component_types.push(component);
        }

        self
    }

    pub fn include_archetype(mut self, archetype: &Archetype) -> Self {
        self.component_types.reserve(self.component_types.capacity() + archetype.component_count());

        for comp in archetype.components_iter() {
            if !self.component_types.contains(&comp) {
                self.component_types.push(comp)
            }
        }

        self
    }

    pub fn build(mut self) -> Archetype {
        self.component_types.sort_by_key(|c| c.id());

        Archetype::new(&mut self.component_types)
    }
}