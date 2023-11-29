use crate::Component;

impl Component for f32 {}
impl Component for u64 {}
impl Component for u128 {}

#[cfg(test)]
mod tests {
    use crate::{Archetype, Store};

    #[test]
    fn registry_set_value_for_entity_as_expected() {
        let entities_capacity = 255;
        let mut store = Store::with_capacity(entities_capacity);
        let archetype = Archetype::with_components::<(u64, u128)>();

        let u64_values: Vec<u64> = (0..entities_capacity).map(|v| v as u64).collect();

        let u128_values: Vec<u128> = (0..entities_capacity)
            .map(|v| (entities_capacity + v) as u128)
            .collect();

        let mut entities = Vec::new();

        for i in 0..entities_capacity {
            let entity = store.create_entity(&archetype);
            entities.push(entity);

            let (u64_v, u128_v) = store.get_components_refs_mut::<(u64, u128)>(entity);
            (*u64_v, *u128_v) = (u64_values[i], u128_values[i]);
        }

        for i in 0..entities_capacity {
            let entity = entities[i];

            let (&actual_u64, &actual_u128) =
                store.get_components_refs::<(u64, u128)>(entity);
            assert_eq!((u64_values[i], u128_values[i]), (actual_u64, actual_u128));
        }
    }

    #[test]
    #[should_panic]
    fn registry_get_component_ref_mut_panics_if_component_doesnt_exist() {
        let mut store = Store::new();
        let archetype = Archetype::with_components::<(u64, u128)>();

        let entity = store.create_entity(&archetype);
        store.get_components_refs_mut::<f32>(entity);
    }

    #[test]
    #[should_panic]
    fn registry_get_component_ref_mut_panics_if_entity_doesnt_alive() {
        let mut registry = Store::new();
        let archetype = Archetype::with_components::<u64>();

        let entity = registry.create_entity(&archetype);
        registry.destroy_entity(entity);

        registry.get_components_refs_mut::<u64>(entity);
    }
}
