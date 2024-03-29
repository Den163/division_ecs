use std::time::Instant;

use division_ecs::{
    query::component, ArchetypeBuilder, Component, ComponentReadOnlyQuery, Store,
};

struct AosObject {
    position: Position,
    rotation: Rotation,
    moving_unit: MovingUnit,
    dirty_data: DirtyData,
}

#[derive(Component, Clone, Copy)]
struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Clone, Copy)]
struct Rotation {
    pub angle: f32,
}

#[derive(Component, Clone, Copy)]
struct MovingUnit {
    pub _speed: f32,
    pub attack: f32,
    pub hit_rate: f32,
}

#[derive(Clone, Copy)]
struct DirtyData {
    pub _x: f32,
    pub _y: f32,
    pub _z: f32,
    pub w: f32,
}

pub const ENTITIES_COUNT: usize = 20_000_000;

#[inline(never)]
pub fn main() {
    let mut registry = Store::new();
    let aos_data = create_data_arrays();

    {
        populate_ecs(&mut registry, &aos_data);
    }

    let begin = Instant::now();
    let aos_result = iterate_oop(&aos_data);
    let delta_time = Instant::now() - begin;

    println!("Array of structs result: {aos_result}. With time: {delta_time:?}");

    let mut query = component::readonly();

    warmup_ecs(&registry, &mut query);

    let begin = Instant::now();
    let ecs_result = iterate_ecs(&registry, &mut query);
    let delta_time = Instant::now() - begin;
    println!("Ecs result: {ecs_result}. With time: {delta_time:?}");

    let last_w = aos_data[ENTITIES_COUNT - 1].dirty_data.w;
    println!("Last dirty data w: {last_w}");
}

#[inline(never)]
fn create_data_arrays() -> Vec<Box<AosObject>> {
    let mut data = Vec::with_capacity(ENTITIES_COUNT);

    for _ in 0..ENTITIES_COUNT {
        data.push(Box::new(AosObject {
            position: Position {
                x: rand::random(),
                y: rand::random(),
            },
            rotation: Rotation {
                angle: rand::random(),
            },
            moving_unit: MovingUnit {
                _speed: rand::random(),
                attack: rand::random(),
                hit_rate: rand::random(),
            },
            dirty_data: DirtyData {
                _x: rand::random(),
                _y: rand::random(),
                _z: rand::random(),
                w: rand::random(),
            },
        }));
    }

    data
}

#[inline(never)]
fn warmup_ecs(
    registry: &Store,
    query: &mut ComponentReadOnlyQuery<(Position, Rotation, MovingUnit)>,
) {
    let mut result = 0u32;

    for (e, _) in registry.component_query_iter(query).with_entities() {
        result = result.wrapping_add(e.id());
    }

    println!("Ecs ids sum: {result}");
}

#[inline(never)]
fn populate_ecs(registry: &mut Store, data: &Vec<Box<AosObject>>) {
    let pos_rot_arch = ArchetypeBuilder::new()
        .include_components::<(Position, Rotation, MovingUnit)>()
        .build();

    for d in data {
        let e = registry.create_entity_with_archetype(&pos_rot_arch);

        let (pos, rot, unit) = registry
            .get_components_refs_mut::<(Position, Rotation, MovingUnit)>(e)
            .unwrap();

        (*pos, *rot, *unit) = (d.position, d.rotation, d.moving_unit);
    }
}

#[inline(never)]
fn iterate_ecs(
    registry: &Store,
    query: &mut ComponentReadOnlyQuery<(Position, Rotation, MovingUnit)>,
) -> f32 {
    let mut result = 0.;
    let mut counter = 0;

    for (pos, rot, moving_unit) in registry.component_query_iter(query) {
        result += test_op(pos, rot, moving_unit);
        counter += 1;
    }

    result / counter as f32
}

#[inline(never)]
fn iterate_oop(oops: &Vec<Box<AosObject>>) -> f32 {
    let mut result = 0.;
    let mut counter = 0;

    for obj in oops {
        result += test_op(&obj.position, &obj.rotation, &obj.moving_unit);
        counter += 1;
    }

    result / counter as f32
}

fn test_op(pos: &Position, rot: &Rotation, un: &MovingUnit) -> f32 {
    (pos.x + pos.y) * rot.angle * (un.attack * un.hit_rate)
}
