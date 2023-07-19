use division_ecs::{component_types, ArchetypeBuilder, ComponentType, Registry, ComponentsReadQuery, QueryIterator};

struct AosObject {
    position: Box<Position>,
    rotation: Box<Rotation>,
    moving_unit: Box<MovingUnit>,
    dirty_data: Box<DirtyData>
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
    pub _x: f32,
    pub _y: f32,
    pub _z: f32,
    pub w: f32
}

#[derive(Clone, Copy)]
struct MovingUnit {
    pub _speed: f32,
    pub attack: f32,
    pub hit_rate: f32
}

pub const ENTITIES_COUNT: usize = 100_000;

pub fn main() {
    let mut registry = Registry::new();
    let aos_data = create_data_arrays();

    {
        populate_ecs(&mut registry, &aos_data);
    }

    let aos_result = iterate_oop(&aos_data);
    println!("Array of structs result: {aos_result}");

    let mut query = create_query(&registry);

    warmup_ecs(&registry, &mut query);

    let ecs_result = iterate_ecs(&registry, &mut query);
    println!("Ecs result: {ecs_result}");
}

#[inline(never)]
fn create_data_arrays() -> Vec<Box<AosObject>> {
    let mut data = Vec::with_capacity(ENTITIES_COUNT);

    for _ in 0..ENTITIES_COUNT {
        data.push(Box::new(AosObject {
            position: Box::new(Position { x: rand::random(), y: rand::random() }),
            rotation: Box::new(Rotation { angle: rand::random() }),
            moving_unit: Box::new(MovingUnit { _speed: rand::random(), attack: rand::random(), hit_rate: rand::random() }),
            dirty_data: Box::new(DirtyData { _x: rand::random(), _y: rand::random(), _z: rand::random(), w: rand::random() })
        }));
    }

    data
}

#[inline(never)]
fn create_query(registry: &Registry) -> ComponentsReadQuery<(Position, Rotation, MovingUnit)> {
    registry.read_query::<(Position, Rotation, MovingUnit)>()
}

#[inline(never)]
fn warmup_ecs(registry: &Registry, query: &mut ComponentsReadQuery<(Position, Rotation, MovingUnit)>) {
    let mut result = 0u32;

    for (e, (_, _, _)) in registry.iter(query) {
        result = result.wrapping_add(e.id());
    }

    println!("Ecs ids sum: {result}");
}

#[inline(never)]
fn populate_ecs(registry: &mut Registry, data: &Vec<Box<AosObject>>) {
    let pos_rot_arch = ArchetypeBuilder::new()
        .component_types(&component_types!(Position, Rotation, MovingUnit))
        .build();

    for d in data {
        let e = registry.create_entity(&pos_rot_arch);
        *registry.get_component_ref_mut(e) = *d.position;
        *registry.get_component_ref_mut(e) = *d.rotation;
        *registry.get_component_ref_mut(e) = *d.moving_unit;
    }
}

#[inline(never)]
fn iterate_ecs(registry: &Registry, query: &mut ComponentsReadQuery<(Position, Rotation, MovingUnit)>) -> f32 {
    let mut result = 0.;
    let mut counter = 0;

    for (_, (pos, rot, moving_unit)) in registry.iter(query) {
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
        result += test_op(&obj.position, &obj.rotation, &obj.moving_unit) * obj.dirty_data.w;
        counter += 1;
    }

    result / counter as f32
}

fn test_op(pos: &Position, rot: &Rotation, un: &MovingUnit) -> f32 {
    (pos.x + pos.y) * rot.angle * (un.attack * un.hit_rate)
}
