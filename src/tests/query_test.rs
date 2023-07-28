#[cfg(test)]
mod test {
    use crate::{
        Store, 
        ArchetypeBuilder, 
        components_query::{QueryIntoIter, ComponentsReadOnlyQuery}, 
        ComponentsWriteQuery, 
        Component
    };

    #[derive(Debug, PartialEq, Component, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq, Component, Clone, Copy)]
    struct Rotation {
        angle: f64
    }

    #[test]
    fn write_and_read_query_test() {
        let mut store = Store::new();
        let arch0 = ArchetypeBuilder::new()
            .include_components::<(Position, Rotation)>()
            .build();

        let arch1 = ArchetypeBuilder::new()
            .include_components::<(Position, Rotation, u64)>()
            .build();

        let expected_data = [
            (Position { x: 10., y: 20. }, Rotation { angle: 90. }, arch0 ), 
            (Position { x: 0., y: 20. }, Rotation { angle: 180. }, arch1 )
        ];

        let mut entities = Vec::new();

        for (_, _, arch) in &expected_data {
            store.create_entity(arch);
        }

        let mut write_query = ComponentsWriteQuery::<(Position, Rotation)>::new();
        let mut iter_count = 0;
        for (e, (pos, rot)) in store.into_iter(&mut write_query) {
            entities.push(e);

            let (expected_pos, expected_rot, _) =  expected_data[iter_count];

            *pos = expected_pos;
            *rot = expected_rot;

            iter_count += 1;
        }
        assert_eq!(iter_count, expected_data.len());

        let other_arch = ArchetypeBuilder::new()
            .include_components::<(f32, u64)>()
            .build();
        
        store.create_entity(&other_arch);
        store.create_entity(&other_arch);

        let mut read_query = ComponentsReadOnlyQuery::<(Position, Rotation)>::new();
        let mut iter_count = 0;
        for (e, (pos, rot)) in store.into_iter(&mut read_query) {
            iter_count += 1;

            let e_idx = entities.iter().position(|e_check| *e_check == e).unwrap();
            let (expected_pos, expected_rot, _) = expected_data[e_idx];

            assert_eq!(expected_pos, *pos);
            assert_eq!(expected_rot, *rot);
        }

        assert_eq!(iter_count, expected_data.len());
    }
}