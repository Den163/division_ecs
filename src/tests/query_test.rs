#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{Registry, ArchetypeBuilder, ComponentType, component_types};

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Rotation {
        angle: f64
    }

    #[test]
    fn read_query_test() {
        let mut registry = Registry::new();
        let arch0 = ArchetypeBuilder::new()
            .component_types(&component_types!(Position, Rotation))
            .build();

        let arch1 = ArchetypeBuilder::new()
            .component_types(&component_types!(Position, Rotation, u64))
            .build();

        let expected_data = vec![
            (Position { x: 10., y: 20. }, Rotation { angle: 90. }, arch0 ), 
            (Position { x: 0., y: 20. }, Rotation { angle: 180. }, arch1 )
        ];
        let mut entities = Vec::new();

        for (pos, rot, arch) in &expected_data {
            let e = registry.create_entity(&arch);
            entities.push(e);
            
            *registry.get_component_ref_mut(e) = *pos;
            *registry.get_component_ref_mut(e) = *rot;
        }

        let other_arch = ArchetypeBuilder::new()
            .component_types(&component_types!(f32, u64))
            .build();
        
        registry.create_entity(&other_arch);
        registry.create_entity(&other_arch);

        let query = &mut registry.read_query::<Position, Rotation>();
        let mut iter_count = 0;
        for (e, pos, rot) in query {
            iter_count += 1;

            let e_idx = entities.iter().position(|e_check| *e_check == e).unwrap();
            let (expected_pos, expected_rot, _) = expected_data[e_idx];

            assert_eq!(expected_pos, *pos);
            assert_eq!(expected_rot, *rot);
        }

        assert_eq!(iter_count, expected_data.len());
    }
}