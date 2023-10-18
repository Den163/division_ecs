#[cfg(test)]
mod tests {
    use crate::{ArchetypeBuilder, Component};

    #[derive(Component, Clone, Copy)]
    struct TestType1;

    #[derive(Component, Clone, Copy)]
    struct TestType2;

    #[derive(Component, Clone, Copy)]
    struct TestType3;

    #[test]
    fn archetype_has_component_when_it_build_with_it() {
        let archetype = ArchetypeBuilder::new()
            .include_components::<(TestType1, TestType3)>()
            .build();

        assert_eq!(archetype.component_count(), 2);
        assert!(archetype.has_component::<TestType1>());
        assert!(archetype.has_component::<TestType2>() == false);
        assert!(archetype.has_component::<TestType3>());
    }

    #[test]
    fn archetype_is_same_as_true_only_with_same_set_of_types() {
        let a1 = ArchetypeBuilder::new()
            .include_components::<(TestType1, TestType2)>()
            .build();

        let a2 = a1.clone();

        assert!(a1.is_same_as(&a2));

        let a3 = ArchetypeBuilder::new()
            .include_components::<(TestType1, TestType3)>()
            .build();

        assert!(a1.is_same_as(&a3) == false);
    }

    #[test]
    fn archetype_include_as_expected() {
        let arch_to_extend = ArchetypeBuilder::new()
            .include_components::<(TestType3, u64)>()
            .build();

        let arch = ArchetypeBuilder::new()
            .include_components::<(TestType1, TestType2)>()
            .include_archetype(&arch_to_extend)
            .build();

        assert!(arch.is_include(&arch_to_extend));

        let not_extended_archetype =
            ArchetypeBuilder::new().include_components::<f32>().build();

        assert!(arch.is_include(&not_extended_archetype) == false);
    }

    #[test]
    fn archetype_exclude_as_expected() {
        let arch = ArchetypeBuilder::new()
            .include_components::<(TestType1, u64)>()
            .build();

        let arch_to_be_excluded = ArchetypeBuilder::new()
            .include_components::<(TestType2, TestType3)>()
            .build();

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
}
