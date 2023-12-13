#[cfg(test)]
mod tests {
    use crate::{query, Component, Entity, Store, Tag};

    #[derive(Tag)]
    struct TestGroup;

    #[derive(Component, Clone, Copy, PartialEq, Debug)]
    struct TestComponent1 {
        val: f32,
    }

    #[derive(Component, Clone, Copy, PartialEq, Debug)]
    struct TestComponent2 {
        val: usize,
    }

    #[test]
    fn empty_query_will_not_iterate() {
        let store = Store::new();
        let mut query = query::ordered_component::readonly::<TestGroup, TestComponent1>();

        assert_eq!(store.ordered_query_iter(&mut query).count(), 0);
    }

    #[test]
    fn query_will_iterate_with_expected_components_order() {
        let mut store = Store::new();

        let mut expected: Vec<(Entity, TestComponent1)> = (0..10)
            .map(|i| {
                let e = store.create_entity();
                let c = TestComponent1 { val: i as f32 };
                store.add_entity_order_by::<TestGroup>(e);
                store.add_components(e, c);

                (e, c)
            })
            .collect();

        store.destroy_entity(expected.remove(0).0);
        store.destroy_entity(expected.remove(2).0);
        store.destroy_entity(expected.remove(expected.len() - 1).0);

        let expected: Vec<TestComponent1> = expected.iter().map(|(_, c)| *c).collect();

        let mut query = query::ordered_component::readonly::<TestGroup, TestComponent1>();
        let iter = store.ordered_query_iter(&mut query);
        let iter_len = iter.len();
        assert_eq!(iter_len, expected.len());

        let actual: Vec<TestComponent1> = iter.cloned().collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn queries_with_different_components_set_will_filter_entities_not_existed_in_iterated_archetype(
    ) {
        let mut store = Store::new();

        let mut expected1 = (0..2)
            .map(|i| create_test_comp1(&mut store, i))
            .collect::<Vec<_>>();
        let mut expected2 = (0..3)
            .map(|i| create_test_comp2(&mut store, i))
            .collect::<Vec<_>>();

        let len1 = expected1.len() - 1;
        let len2 = expected2.len() - 1;

        expected1.extend((len1..len1 + 3).map(|i| create_test_comp1(&mut store, i)));
        expected2.extend((len2..len2 + 5).map(|i| create_test_comp2(&mut store, i)));

        let mut query1 =
            query::ordered_component::readonly::<TestGroup, TestComponent1>();
        let mut query2 =
            query::ordered_component::readonly::<TestGroup, TestComponent2>();

        let actual1 = store
            .ordered_query_iter(&mut query1)
            .cloned()
            .collect::<Vec<_>>();
        let actual2 = store
            .ordered_query_iter(&mut query2)
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(expected1, actual1);
        assert_eq!(expected2, actual2);
    }

    fn create_test_comp1(store: &mut Store, i: usize) -> TestComponent1 {
        let e = store.create_entity();
        let c = TestComponent1 { val: i as f32 };

        store.add_entity_order_by::<TestGroup>(e);
        store.add_components(e, c);

        c
    }

    fn create_test_comp2(store: &mut Store, i: usize) -> TestComponent2 {
        let e = store.create_entity();
        let c = TestComponent2 { val: i };

        store.add_entity_order_by::<TestGroup>(e);
        store.add_components(e, c);

        c
    }

    #[test]
    fn with_entities_iter_as_expected() {
        let mut store = Store::new();

        let expected: Vec<_> = (0..5)
            .map(|i| {
                let e = store.create_entity();
                let c = TestComponent1 { val: i as f32 };

                store.add_entity_order_by::<TestGroup>(e);
                store.add_components(e, c);

                (e, c)
            })
            .collect();

        let mut query = query::ordered_component::readonly::<TestGroup, TestComponent1>();
        let actual: Vec<_> = store
            .ordered_query_iter(&mut query)
            .with_entities()
            .map(|(e, c)| (e, *c))
            .collect();

        assert_eq!(expected, actual)
    }
}
