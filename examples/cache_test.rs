use division_ecs::{component_types, ArchetypeBuilder, ComponentType, Registry};

struct Position {
    pub x: f32,
    pub y: f32,
}

struct Rotation {
    pub angle: f32,
}

pub fn main() {
    let mut registry = Registry::new();

    populate(&mut registry);
    let result = iterate(&registry);

    println!("Result: {result}");
}

#[inline(never)]
fn populate(registry: &mut Registry) {
    let pos_rot_arch = ArchetypeBuilder::new()
        .component_types(&component_types!(Position, Rotation))
        .build();

    for _ in 0..10000 {
        let e = registry.create_entity(&pos_rot_arch);
        *registry.get_component_ref_mut(e) = Position {
            x: rand::random(),
            y: rand::random(),
        };

        *registry.get_component_ref_mut(e) = Rotation {
            angle: rand::random(),
        };
    }
}

#[inline(never)]
fn iterate(registry: &Registry) -> f32 {
    let mut query = registry.read_query::<(Position, Rotation)>();
    let mut result = 0.;

    for (_, (pos, rot)) in &mut query {
        result += (pos.x + pos.y) * rot.angle;
    }

    result
}