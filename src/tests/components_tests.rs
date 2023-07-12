#[cfg(test)]
mod tests {
    use crate::{ArchetypeBuilder, Registry};

    #[test]
    fn registry_set_value_for_entity_as_expected() {
        let entities_capacity = 255;
        let mut registry = Registry::with_capacity(entities_capacity);
        let archetype = ArchetypeBuilder::new()
            .component::<u64>()
            .component::<u128>()
            .build();

        let u64_values: Vec<u64> = (0..entities_capacity).map(|v| v as u64).collect();

        let u128_values: Vec<u128> = (0..entities_capacity)
            .map(|v| (entities_capacity + v) as u128)
            .collect();

        let mut entities = Vec::new();

        for i in 0..entities_capacity {
            let entity = registry.create_entity(&archetype);
            entities.push(entity);

            {
                *registry.get_component_ref_mut(entity) = u64_values[i];
            }

            {
                *registry.get_component_ref_mut(entity) = u128_values[i];
            }
        }

        for i in 0..entities_capacity {
            let entity = entities[i];

            assert_eq!(u64_values[i], *registry.get_component_ref(entity));
            assert_eq!(u128_values[i], *registry.get_component_ref(entity));
        }
    }

    #[test]
    #[should_panic]
    fn registry_get_component_ref_mut_panics_if_component_doesnt_exist() {
        let mut registry = Registry::new();
        let archetype = ArchetypeBuilder::new()
            .component::<u64>()
            .component::<u128>()
            .build();

        let entity = registry.create_entity(&archetype);

        registry.get_component_ref_mut::<f32>(entity);
    }

    #[test]
    #[should_panic]
    fn registry_get_component_ref_mut_panics_if_entity_doesnt_alive() {
        let mut registry = Registry::new();
        let archetype = ArchetypeBuilder::new().component::<u64>().build();

        let entity = registry.create_entity(&archetype);
        registry.destroy_entity(entity);

        registry.get_component_ref_mut::<u64>(entity);
    }
}
