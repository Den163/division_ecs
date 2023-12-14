#[cfg(test)]
mod tests {
    use crate::{type_ids, Archetype, Component, Store};
    use std::mem::MaybeUninit;

    impl Component for f32 {}
    impl Component for u64 {}
    impl Component for u128 {}

    #[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
    struct TestComponent1 {
        value: i32,
    }

    #[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
    struct TestComponent2 {
        value: f64,
    }

    impl TestComponent1 {
        pub fn new(i: usize) -> TestComponent1 {
            TestComponent1 { value: i as i32 }
        }
    }

    impl TestComponent2 {
        pub fn new(i: usize) -> TestComponent2 {
            TestComponent2 {
                value: (i * 2) as f64,
            }
        }
    }

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
            let entity = store.create_entity_with_archetype(&archetype);
            entities.push(entity);

            let (u64_v, u128_v) = store
                .get_components_refs_mut::<(u64, u128)>(entity)
                .unwrap();
            (*u64_v, *u128_v) = (u64_values[i], u128_values[i]);
        }

        for i in 0..entities_capacity {
            let entity = entities[i];

            let (&actual_u64, &actual_u128) =
                store.get_components_refs::<(u64, u128)>(entity).unwrap();
            assert_eq!((u64_values[i], u128_values[i]), (actual_u64, actual_u128));
        }
    }

    #[test]
    fn registry_get_component_ref_mut_is_none_if_component_doesnt_exist() {
        let mut store = Store::new();
        let archetype = Archetype::with_components::<(u64, u128)>();

        let entity = store.create_entity_with_archetype(&archetype);
        assert!(store.get_components_refs_mut::<f32>(entity).is_none());
    }

    #[test]
    fn registry_get_component_ref_mut_is_none_if_entity_doesnt_alive() {
        let mut registry = Store::new();
        let archetype = Archetype::with_components::<u64>();

        let entity = registry.create_entity_with_archetype(&archetype);
        registry.destroy_entity(entity);

        assert!(registry.get_components_refs_mut::<u64>(entity).is_none());
    }

    #[test]
    fn registry_destroy_component_after_internal_page_swap_remove_as_expected() {
        let mut registry = Store::new();
        let archetype = Archetype::with_components::<(TestComponent1, TestComponent2)>();

        let mut entity_to_swap_remove = MaybeUninit::uninit();
        let mut swapped_entity = MaybeUninit::uninit();
        for i in 0..4 {
            let e = registry.create_entity_with_archetype(&archetype);
            let (c1, c2) = registry
                .get_components_refs_mut::<(TestComponent1, TestComponent2)>(e)
                .unwrap();

            (*c1, *c2) = (TestComponent1::new(i), TestComponent2::new(i));

            match i {
                1 => {
                    entity_to_swap_remove.write(e);
                }
                3 => {
                    swapped_entity.write((e, *c1, *c2));
                }
                _ => {}
            }
        }

        let entity_to_swap_remove = unsafe { entity_to_swap_remove.assume_init() };
        let swapped_entity = unsafe { swapped_entity.assume_init() };
        let (swapped_entity, expected1, expected2) = swapped_entity;

        registry.destroy_entity(entity_to_swap_remove);

        let (actual1, actual2) = registry
            .get_components_refs_mut::<(TestComponent1, TestComponent2)>(swapped_entity)
            .unwrap();

        assert_eq!(actual1.value, expected1.value);
        assert_eq!(actual2.value, expected2.value);
    }

    #[test]
    fn add_components_to_empty_archetype_changes_it_respectively() {
        let mut store = Store::new();
        let e = store.create_entity();

        store.add_components(
            e,
            (TestComponent1 { value: 0 }, TestComponent2 { value: 0. }),
        );

        let arch = store.get_entity_archetype(e).unwrap();

        assert!(arch.is_include_only_ids(&type_ids!(TestComponent1, TestComponent2)));
    }

    #[test]
    fn add_components_values_as_expected() {
        let mut store = Store::new();
        let e = store.create_entity();

        let expected1 = TestComponent1 { value: 364 };
        let expected2 = TestComponent2 { value: 31. };

        store.add_components(e, expected1);
        store.add_components(e, expected2);

        let (actual1, actual2) = store
            .get_components_refs::<(TestComponent1, TestComponent2)>(e)
            .unwrap();

        assert_eq!(*actual1, expected1);
        assert_eq!(*actual2, expected2);
    }

    #[test]
    fn add_components_with_intersected_types_as_expected() {
        let mut store = Store::new();
        let e0 = store.create_entity();
        let e1 = store.create_entity();

        let expected00 = TestComponent1::new(1);
        let expected01 = 15u64;

        let expected10 = TestComponent2::new(15);
        let expected11 = TestComponent1::new(2);

        store.add_components(e0, expected00);
        store.add_components(e1, expected10);
        store.add_components(e0, expected01);
        store.add_components(e1, expected11);

        let (actual00, actual01) = store
            .get_components_refs::<(TestComponent1, u64)>(e0)
            .unwrap();
        let (actual10, actual11) = store
            .get_components_refs::<(TestComponent2, TestComponent1)>(e1)
            .unwrap();

        assert_eq!(*actual00, expected00);
        assert_eq!(*actual01, expected01);
        assert_eq!(*actual10, expected10);
        assert_eq!(*actual11, expected11);
    }

    #[test]
    fn add_components_values_work_with_different_archetypes() {
        let mut store = Store::new();
        let e1 = store.create_entity();

        let expected1 = TestComponent1 { value: 364 };
        store.add_components(e1, expected1);

        let e2 = store.create_entity();

        let expected2 = TestComponent2 { value: 31. };
        store.add_components(e2, expected2);

        let actual1 = store.get_components_refs::<TestComponent1>(e1).unwrap();
        let actual2 = store.get_components_refs::<TestComponent2>(e2).unwrap();

        assert!(store
            .get_entity_archetype(e1)
            .unwrap()
            .is_include_only_ids(&type_ids!(TestComponent1)),);

        assert!(store
            .get_entity_archetype(e2)
            .unwrap()
            .is_include_only_ids(&type_ids!(TestComponent2)));

        assert_eq!(*actual1, expected1);
        assert_eq!(*actual2, expected2);
    }

    #[test]
    fn remove_components_archetype_will_not_contain_erased_types() {
        let mut store = Store::new();
        let entity = store.create_entity();

        store.add_components(
            entity,
            (TestComponent1::default(), TestComponent2::default()),
        );
        store.remove_components::<TestComponent2>(entity);

        let arch = store.get_entity_archetype(entity).unwrap();

        assert!(arch.is_exclude_ids(&type_ids!(TestComponent2)));
        assert!(arch.is_include_ids(&type_ids!(TestComponent1)));
    }

    #[test]
    fn remove_components_archetype_values_as_expected() {
        let mut store = Store::new();
        let entity = store.create_entity();
        let expected_comp = TestComponent2::new(33);

        store.add_components(entity, (TestComponent1::default(), expected_comp));
        store.remove_components::<TestComponent1>(entity);

        let comp2 = store.get_components_refs::<TestComponent2>(entity).unwrap();

        assert_eq!(*comp2, expected_comp);
    }
}
