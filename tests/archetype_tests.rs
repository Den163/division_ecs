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
    let entities_capacity = layout.entities_capacity();

    assert!(entities_capacity > 0);

    let u64_values: Vec<u64> = (0..entities_capacity)
        .map(|v| { v as u64 })
        .collect();

    let u128_values: Vec<u128> = (0..entities_capacity)
        .map(|v| { (entities_capacity + v) as u128 })
        .collect();

    for i in 0..entities_capacity {
        page.set_component_value::<u64>(i, &archetype, &layout, u64_values[i]);
        page.set_component_value::<u128>(i, &archetype, &layout, u128_values[i]);
    }

    for i in 0..entities_capacity {
        assert_eq!(u64_values[i], *page.get_component_value::<u64>(i, &archetype, &layout));
        assert_eq!(u128_values[i], *page.get_component_value::<u128>(i, &archetype, &layout));
    }
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
fn archetype_extends_as_expected() {
    let arch_to_extend = ArchetypeBuilder::new()
        .component::<TestType3>()
        .component::<u64>()
        .build();

    let arch = ArchetypeBuilder::new()
        .component::<TestType1>()
        .component::<TestType2>()
        .extend_archetype(&arch_to_extend)
        .build();

    assert!(arch.is_extends(&arch_to_extend));
}