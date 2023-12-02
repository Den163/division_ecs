#[cfg(test)]
mod test {
    use crate::{Archetype, Component, Entity, EntityComponentReadOnlyQuery, Store};

    #[derive(Component, Clone, Copy)]
    struct TestComponent1 {
        _x: f32,
        _y: u128
    }

    #[derive(Component, Clone, Copy)]
    struct TestComponent2 {
        value: f64,
    }

    #[test]
    fn entity_component_query_iter_order_as_expected() {
        let mut store = Store::new();
        let mut entities = Vec::new();
        let mut component_values = Vec::new();
        let archetype1 = Archetype::with_components::<(TestComponent1, TestComponent2)>();
        let archetype2 = Archetype::with_components::<TestComponent2>();
        let arch1_entity_count = 51;
        let arch2_entity_count = 48;

        let last_index_after_first_destroy = arch1_entity_count + arch2_entity_count - 2;
        let entities_indices_to_destroy = [0, last_index_after_first_destroy, 32, 72];

        for _ in 0..arch1_entity_count {
            create_entity_with_component(
                &mut store,
                &archetype1,
                &mut entities,
                &mut component_values,
            );
        }

        for _ in 0..arch2_entity_count {
            create_entity_with_component(
                &mut store,
                &archetype2,
                &mut entities,
                &mut component_values,
            );
        }

        for i in entities_indices_to_destroy {
            destroy_entity_with_component(
                &mut store,
                i,
                &mut entities,
                &mut component_values,
            );
        }

        let mut query =
            EntityComponentReadOnlyQuery::<TestComponent2>::with_entities(&entities);

        let mut i = 0;
        for (e, (c,)) in store.entity_component_query_iter(&mut query) {
            assert_eq!(e, entities[i]);
            assert_eq!(c.value, component_values[i].value);

            i += 1;
        }

        assert_eq!(entities.len(), i);
    }

    fn create_entity_with_component(
        store: &mut Store,
        archetype: &Archetype,
        entities: &mut Vec<Entity>,
        component_values: &mut Vec<TestComponent2>,
    ) {
        let e = store.create_entity_with_archetype(&archetype);
        let (comp,) = store.get_components_refs_mut::<TestComponent2>(e);

        *comp = TestComponent2 { value: e.id as f64 };

        entities.push(e);
        component_values.push(*comp);
    }

    fn destroy_entity_with_component(
        store: &mut Store,
        index: usize,
        entities: &mut Vec<Entity>,
        component_values: &mut Vec<TestComponent2>,
    ) {
        store.destroy_entity(entities.remove(index));
        component_values.remove(index);
    }
}
