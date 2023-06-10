use division_ecs::{Registry, ArchetypeBuilder};

struct TestType1;
struct TestType2;
struct TestType3;

#[test]
fn archetype_has_component_when_it_build_with_it() {
    let archetype = ArchetypeBuilder::new()
        .component::<TestType1>()
        .component::<TestType3>()
        .build();

    assert!(archetype.has_component::<TestType1>());
    assert!(archetype.has_component::<TestType2>() == false);
    assert!(archetype.has_component::<TestType3>());
}