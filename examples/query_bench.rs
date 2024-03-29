use std::time::Instant;

use division_ecs::{query, Component, Store, Tag};
use rand::random;

#[derive(Component, Clone, Copy)]
struct TestComponent1 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component, Clone, Copy)]
struct TestComponent2 {
    xx: f64,
    yy: f32,
    zz: f64,
}

#[derive(Tag)]
struct TestTag;

const ENTITY_COUNT: usize = 2_000_000;

fn main() {
    let mut store = Store::new();

    for i in 0..ENTITY_COUNT {
        let e = store.create_entity();
        store.add_components(
            e,
            (
                TestComponent1 {
                    x: random(),
                    y: random(),
                    z: random(),
                },
                TestComponent2 {
                    xx: random(),
                    yy: random(),
                    zz: random(),
                },
            ),
        );

        if i % 10 != 0 {
            store.add_entity_order_by::<TestTag>(e);
            store.add_tag::<TestTag>(e);
        }
    }

    let mut query = query::component::readonly::<(TestComponent1, TestComponent2)>();

    let mut result = 0.;
    let begin = Instant::now();

    for (_, (comp1, comp2)) in store
        .component_query_iter(&mut query)
        .with_entities()
        .filter_tag::<TestTag>()
    {
        result_func(&mut result, comp1, comp2);
    }

    let delta_time = Instant::now() - begin;

    println!("Query with tags: {result}. With time: {delta_time:?}");

    let mut query =
        query::ordered_component::readonly::<TestTag, (TestComponent1, TestComponent2)>();

    let mut result = 0.;
    let begin = Instant::now();

    for (_, (comp1, comp2)) in store.ordered_query_iter(&mut query).with_entities() {
        result_func(&mut result, comp1, comp2);
    }

    let delta_time = Instant::now() - begin;

    println!("Query with order group: {result}. With time: {delta_time:?}");
}

#[inline]
fn result_func(result: &mut f64, comp1: &TestComponent1, comp2: &TestComponent2) {
    *result += (comp1.x as f64 * comp2.xx)
        + (comp1.y as f64 * comp2.yy as f64)
        + (comp1.z as f64 * comp2.zz);
}
