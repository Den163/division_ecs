use division_ecs::{component_types, ArchetypeBuilder, ComponentType, Registry, ComponentsReadQuery};

#[derive(Clone, Copy)]
struct AosObject {
    position: Position,
    rotation: Rotation,
    moving_unit: MovingUnit,
    dirty_data: DirtyData
}

#[derive(Clone, Copy)]
struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
struct Rotation {
    pub angle: f32,
}

#[derive(Clone, Copy)]
struct DirtyData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

#[derive(Clone, Copy)]
struct MovingUnit {
    pub speed: f32,
    pub attack: f32,
    pub hit_rate: f32
}

pub const ENTITIES_COUNT: usize = 1_000_000;

pub fn main() {
    let mut registry = Registry::new();
    let aos_data = create_data_arrays();

    {
        populate_ecs(&mut registry, &aos_data);
    }

    let aos_result = iterate_aos(&aos_data);
    println!("Array of structs result: {aos_result}");

    let mut query = create_query(&registry);

    {
        let ecs_result = iterate_ecs(&mut query);
        println!("Ecs result: {ecs_result}");
    }
}

#[inline(never)]
fn create_data_arrays() -> Vec<AosObject> {
    let mut data = Vec::with_capacity(ENTITIES_COUNT);

    for i in 0..ENTITIES_COUNT {
        data.push(AosObject {
            position: Position { x: rand::random(), y: rand::random() },
            rotation: Rotation { angle: rand::random() },
            moving_unit: MovingUnit { speed: rand::random(), attack: rand::random(), hit_rate: rand::random() },
            dirty_data: DirtyData { x: rand::random(), y: rand::random(), z: rand::random(), w: rand::random() }
        })
    }

    data
}

#[inline(never)]
fn create_query(registry: &Registry) -> ComponentsReadQuery<(Position, Rotation, MovingUnit)> {
    registry.read_query::<(Position, Rotation, MovingUnit)>()
}

#[inline(never)]
fn warmup_ecs<'a>(query: &'a mut ComponentsReadQuery<'a, (Position, Rotation, MovingUnit)>) {
    let mut result = 0u32;

    for (e, (poos, rot, mov)) in query {
        result = result.wrapping_add(e.id());
    }

    println!("Ecs ids sum: {result}");
}

#[inline(never)]
fn populate_ecs(registry: &mut Registry, data: &Vec<AosObject>) {
    let pos_rot_arch = ArchetypeBuilder::new()
        .component_types(&component_types!(Position, Rotation, MovingUnit))
        .build();

    for d in data {
        let e = registry.create_entity(&pos_rot_arch);
        *registry.get_component_ref_mut(e) = d.position;
        *registry.get_component_ref_mut(e) = d.rotation;
        *registry.get_component_ref_mut(e) = d.moving_unit;
    }
}

#[inline(never)]
fn iterate_ecs<'a>(query: &'a mut ComponentsReadQuery<'a, (Position, Rotation, MovingUnit)>) -> f32 {
    let mut result = 0.;
    let mut counter = 0;

    for (_, (pos, rot, moving_unit)) in query {
        result += test_op(pos, rot, moving_unit);
        counter += 1;
    }

    result / counter as f32
}

#[inline(never)]
fn iterate_aos(oops: &Vec<AosObject>) -> f32 {
    let mut result = 0.;
    let mut counter = 0;

    for obj in oops {
        result += test_op(&obj.position, &obj.rotation, &obj.moving_unit) * obj.dirty_data.w;
        counter += 1;
    }

    result / counter as f32
}

fn test_op(pos: &Position, rot: &Rotation, un: &MovingUnit) -> f32 {
    (pos.x + pos.y) * rot.angle * (un.attack * un.hit_rate)
}
