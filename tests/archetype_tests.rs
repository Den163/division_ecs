use std::{u128};

use division_ecs::{
    ArchetypeBuilder, 
    archetype_data_layout::ArchetypeDataLayout,
    archetype_data_page::ArchetypeDataPage
};

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

#[test]
fn archetype_data_page_set_value_as_expected() {
    let archetype = ArchetypeBuilder::new()
        .component::<u64>()
        .component::<u128>()
        .build();

    let layout = ArchetypeDataLayout::new(&archetype);
    let mut page = ArchetypeDataPage::new();
    let expected_value_u64 = 13;
    let expected_value_u128 = 7;
    let entity_index = 0;

    page.set_component_value::<u64>(entity_index, &archetype, &layout, expected_value_u64);
    page.set_component_value::<u128>(entity_index, &archetype, &layout, expected_value_u128);

    assert_eq!(expected_value_u64, *page.get_component_value::<u64>(entity_index, &archetype, &layout));
    assert_eq!(expected_value_u128, *page.get_component_value::<u128>(entity_index, &archetype, &layout));
}

#[test]
#[should_panic]
fn archetype_data_page_set_value_panics_if_component_doesnt_exist() {
    let archetype = ArchetypeBuilder::new()
        .component::<u64>()
        .component::<u128>()
        .build();

    let layout = ArchetypeDataLayout::new(&archetype);
    let mut page = ArchetypeDataPage::new();

    page.set_component_value::<f32>(0, &archetype, &layout, 12.);
}

#[test]
#[should_panic]
fn archetype_data_page_set_value_panics_if_entity_index_out_of_capacity() {
    let archetype = ArchetypeBuilder::new()
        .component::<u64>()
        .build();

    let layout = ArchetypeDataLayout::new(&archetype);
    let mut page = ArchetypeDataPage::new();

    page.set_component_value::<u64>(layout.entities_capacity(), &archetype, &layout, 0);
}