use division_ecs::ArchetypeBuilder;

struct TestType1;
struct TestType2;
struct TestType3;

#[test]
fn archetype_has_component_when_it_build_with_it() {
    let archetype = ArchetypeBuilder::new()
        .component::<TestType1>()
        .component::<TestType3>()
        .build();

    assert_eq!(archetype.component_count(), 2);
    assert!(archetype.has_component::<TestType1>());
    assert!(archetype.has_component::<TestType2>() == false);
    assert!(archetype.has_component::<TestType3>());
}

#[test]
fn archetype_is_same_as_true_only_with_same_set_of_types() {
    let a1 = ArchetypeBuilder::new()
        .component::<TestType1>()
        .component::<TestType2>()
        .build();

    let a2 = a1.clone();

    assert!(a1.is_same_as(&a2));

    let a3 = ArchetypeBuilder::new()
        .component::<TestType1>()
        .component::<TestType3>()
        .build();

    assert!(a1.is_same_as(&a3) == false);
}

#[test]
fn archetype_include_as_expected() {
    let arch_to_extend = ArchetypeBuilder::new()
        .component::<TestType3>()
        .component::<u64>()
        .build();

    let arch = ArchetypeBuilder::new()
        .component::<TestType1>()
        .component::<TestType2>()
        .include_archetype(&arch_to_extend)
        .build();

    assert!(arch.is_include(&arch_to_extend));

    let not_extended_archetype = ArchetypeBuilder::new()
        .component::<f32>()
        .build();

    assert!(arch.is_include(&not_extended_archetype) == false);
}
