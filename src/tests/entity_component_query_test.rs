#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::{Archetype, Component, Entity, EntityComponentReadOnlyQuery, Store};

    #[derive(Component, Clone, Copy, PartialEq, Debug)]
    struct TestComponent1 {
        x: f32,
        y: u128,
    }

    #[derive(Component, Clone, Copy, PartialEq, Debug)]
    struct TestComponent2 {
        value: f64,
    }

    #[test]
    fn entity_component_query_iter_will_not_iterate_over_invalid_entities() {
        let store = RefCell::new(Store::new());
        let mut valid_entities = Vec::new();
        let mut invalid_entities = Vec::new();

        let valid_arch = Archetype::with_components::<(TestComponent1, TestComponent2)>();
        let invalid_arch = Archetype::with_components::<usize>();

        let create_valid_entities = |i| {
            let mut store = store.borrow_mut();
            let e = store.create_entity_with_archetype(&valid_arch);
            let (comp1, comp2) = store
                .get_components_refs_mut::<(TestComponent1, TestComponent2)>(e)
                .unwrap();

            *comp1 = TestComponent1 {
                x: i as f32,
                y: i as u128 * 2,
            };
            *comp2 = TestComponent2 {
                value: i as f64 * 3.,
            };

            (e, *comp1, *comp2)
        };

        valid_entities.extend((0..7).map(create_valid_entities));

        invalid_entities.extend((7..15).map(|_| {
            store
                .borrow_mut()
                .create_entity_with_archetype(&invalid_arch)
        }));

        valid_entities.extend((15..21).map(create_valid_entities));

        let all_entities: Vec<Entity> = valid_entities
            .iter()
            .map(|(e, _, _)| *e)
            .chain(invalid_entities.into_iter())
            .collect();

        let mut query =
            EntityComponentReadOnlyQuery::<(TestComponent1, TestComponent2)>::new();

        let mut i = 0;
        for (actual_entity, (actual_val1, actual_val2)) in store
            .borrow()
            .entity_component_query_iter(&all_entities, &mut query)
            .with_entities()
        {
            let (expected_entity, expected_val1, expected_val2) = &valid_entities[i];

            assert_eq!(*expected_entity, actual_entity);
            assert_eq!(expected_val1, actual_val1);
            assert_eq!(expected_val2, actual_val2);

            i += 1;
        }

        assert_eq!(i, valid_entities.len())
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

        for i in 0..entities.len() - 1 {
            if i % 3 == 0 {
                let x = i;
                let y = i * 17 % entities.len();
                entities.swap(x, y);
                component_values.swap(x, y);
            }
        }

        let mut query = EntityComponentReadOnlyQuery::<TestComponent2>::new();

        let mut i = 0;
        for (e, c) in store
            .entity_component_query_iter(&entities, &mut query)
            .with_entities()
        {
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
        let comp = store.get_components_refs_mut::<TestComponent2>(e).unwrap();

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
