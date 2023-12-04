#[cfg(test)]
mod tests {
    use crate::{archetype::ArchetypesUnion, Archetype, ArchetypeBuilder, Component};

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
    fn archetype_union_as_expected() {
        {
            let arch0 = Archetype::with_components::<TestType2>();
            let arch1 = Archetype::with_components::<(TestType1, TestType2)>();

            let union = ArchetypesUnion::calculate(&arch0, &arch1);

            assert_union_len(&union, 1);

            assert_type_index::<TestType2>(&arch0, union.lhs_indices[0]);
            assert_type_index::<TestType2>(&arch1, union.rhs_indices[0]);
        }
        {
            let arch0 = Archetype::with_components::<(TestType1, TestType2)>();
            let arch1 = Archetype::with_components::<TestType2>();

            let union = ArchetypesUnion::calculate(&arch0, &arch1);

            assert_union_len(&union, 1);

            assert_type_index::<TestType2>(&arch0, union.lhs_indices[0]);
            assert_type_index::<TestType2>(&arch1, union.rhs_indices[0]);
        }
        {
            let arch0 = Archetype::with_components::<(TestType1, TestType2)>();
            let arch1 = Archetype::with_components::<TestType3>();

            let union = ArchetypesUnion::calculate(&arch0, &arch1);

            assert_union_len(&union, 0);
        }
        {
            let arch0 = Archetype::with_components::<(TestType1, TestType2, TestType3)>();
            let arch1 = Archetype::with_components::<(TestType1, TestType3)>();

            let union = ArchetypesUnion::calculate(&arch0, &arch1);

            assert_union_len(&union, 2);

            assert_type_index::<TestType1>(&arch0, union.lhs_indices[0]);
            assert_type_index::<TestType3>(&arch0, union.lhs_indices[1]);

            assert_type_index::<TestType1>(&arch1, union.lhs_indices[0]);
            assert_type_index::<TestType3>(&arch1, union.lhs_indices[1]);
        }
    }

    fn assert_type_index<T: Component + 'static>(
        archetype: &Archetype,
        expected_type_index: usize,
    ) {
        assert_eq!(
            expected_type_index,
            archetype.find_component_index_of::<T>().unwrap()
        );
    }

    fn assert_union_len(union: &ArchetypesUnion, expected_len: usize) {
        assert_eq!(union.lhs_indices.len(), expected_len);
        assert_eq!(union.rhs_indices.len(), expected_len);
    }
}
