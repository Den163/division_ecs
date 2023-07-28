use crate::{component_type::ComponentType, archetype::Archetype, tuple::ComponentsTuple};

pub trait ArchetypeBuilerTupleExtension : ComponentsTuple {
    fn add_components_to_archetype_builder(builder: &mut ArchetypeBuilder) -> &mut ArchetypeBuilder;
}

pub struct ArchetypeBuilder {
    component_types: Vec<ComponentType>,
}

impl ArchetypeBuilder {
    pub fn new() -> ArchetypeBuilder {
        ArchetypeBuilder { component_types: Vec::new() }
    }

    pub fn include_components<T: ArchetypeBuilerTupleExtension>(&mut self) -> &mut Self {
        T::add_components_to_archetype_builder(self)
    }

    pub fn include_component_types(&mut self, components: &[ComponentType]) -> &mut Self {
        for comp in components {
            self.include_component_type(*comp);
        }

        self
    }

    pub fn include_component_type(&mut self, component: ComponentType) -> &mut Self {
        if !self.component_types.contains(&component) {
            self.component_types.push(component);
        }

        self
    }

    pub fn include_archetype(&mut self, archetype: &Archetype) -> &mut Self {
        self.component_types.reserve(self.component_types.capacity() + archetype.component_count());

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

macro_rules! archetype_builder_tuple_impl {
    ($($T: tt),*) => {
        #[allow(unused_parens)]
        impl<$($T: 'static + Component),*> $crate::archetype_builder::ArchetypeBuilerTupleExtension for ($($T),*) {
            fn add_components_to_archetype_builder(
                builder: &mut $crate::ArchetypeBuilder) -> &mut $crate::ArchetypeBuilder 
            {
                let components = & $crate::component_types!( $($T),* );
                builder.include_component_types(components)
            }
        }
    };
}

pub(crate) use archetype_builder_tuple_impl;