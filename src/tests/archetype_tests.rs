#[cfg(test)]
mod tests {
    use crate::{Archetype, ArchetypeBuilder, Component, Store};

    #[derive(Component, Clone, Copy)]
    struct TestType1 {
        _v: f32,
    }

    #[derive(Component, Clone, Copy)]
    struct TestType2 {
        _v: f32,
    }

    #[derive(Component, Clone, Copy)]
    struct TestType3 {
        _v: f32,
    }

    #[test]
    fn archetype_has_component_when_it_build_with_it() {
        let archetype = Archetype::with_components::<(TestType1, TestType3)>();

        assert_eq!(archetype.component_count(), 2);
        assert!(archetype.has_component::<TestType1>());
        assert!(archetype.has_component::<TestType2>() == false);
        assert!(archetype.has_component::<TestType3>());
    }

    #[test]
    fn archetype_is_same_as_true_only_with_same_set_of_types() {
        let a1 = Archetype::with_components::<(TestType1, TestType2)>();

        let a2 = a1.clone();

        assert!(a1.is_same_as(&a2));

        let a3 = Archetype::with_components::<(TestType1, TestType3)>();

        assert!(a1.is_same_as(&a3) == false);
    }

    #[test]
    fn archetype_include_as_expected() {
        let arch_to_extend = Archetype::with_components::<(TestType3, u64)>();

        let arch = ArchetypeBuilder::new()
            .include_components::<(TestType1, TestType2)>()
            .include_archetype(&arch_to_extend)
            .build();

        assert!(arch.is_include(&arch_to_extend));

        let not_extended_archetype = Archetype::with_components::<f32>();

        assert!(arch.is_include(&not_extended_archetype) == false);
    }

    #[test]
    fn archetype_exclude_as_expected() {
        let arch = Archetype::with_components::<(TestType1, u64)>();
        let arch_to_be_excluded = Archetype::with_components::<(TestType2, TestType3)>();

        assert!(arch.is_exclude(&arch_to_be_excluded));

        let arch_with_one_match_type = ArchetypeBuilder::new()
            .include_archetype(&arch_to_be_excluded)
            .include_components::<u64>()
            .build();

        assert!(arch.is_exclude(&arch_with_one_match_type) == false);

        let arch_with_one_match_type = ArchetypeBuilder::new()
            .include_archetype(&arch_to_be_excluded)
            .include_components::<TestType1>()
            .build();

        assert!(arch.is_exclude(&arch_with_one_match_type) == false);

        let same_types_arch = ArchetypeBuilder::new().include_archetype(&arch).build();

        assert!(arch.is_exclude(&same_types_arch) == false);
    }

    #[test]
    fn store_get_entity_archetype_archetypes_are_same() {
        let arch = Archetype::with_components::<(u64, u32)>();
        let mut store = Store::new();

        let e0 = store.create_entity_with_archetype(&arch);
        let e1 = store.create_entity_with_archetype(&arch);

        assert!(store
            .get_entity_archetype(e0)
            .is_same_as(store.get_entity_archetype(e1)),);
    }

    #[test]
    fn store_get_entity_archetype_archetypes_are_different() {
        let arch0 = Archetype::with_components::<(u64, u32)>();
        let arch1 = Archetype::with_components::<u64>();

        let mut store = Store::new();

        let e0 = store.create_entity_with_archetype(&arch0);
        let e1 = store.create_entity_with_archetype(&arch1);

        let arch0 = store.get_entity_archetype(e0);
        let arch1 = store.get_entity_archetype(e1);

        assert!(arch0.is_same_as(arch1) == false);
    }
}
