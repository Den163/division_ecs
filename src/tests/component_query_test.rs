#[cfg(test)]
mod test {
    use crate::{
        query::component::ComponentReadOnlyQuery, Archetype, Component,
        ComponentWriteQuery, Store,
    };

    #[derive(Debug, PartialEq, Component, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq, Component, Clone, Copy)]
    struct Rotation {
        angle: f64,
    }

    impl Component for usize {}

    #[test]
    fn write_and_read_query_test() {
        let mut store = Store::new();
        let arch0 = Archetype::with_components::<(Position, Rotation)>();
        let arch1 = Archetype::with_components::<(Position, Rotation, u64)>();

        let expected_data = [
            (Position { x: 10., y: 20. }, Rotation { angle: 90. }, arch0),
            (Position { x: 0., y: 20. }, Rotation { angle: 180. }, arch1),
        ];

        let mut entities = Vec::new();

        for (_, _, arch) in &expected_data {
            store.create_entity_with_archetype(arch);
        }

        let mut query = ComponentWriteQuery::<(Position, Rotation)>::new();
        let mut iter_count = 0;
        for (e, (pos, rot)) in store.component_query_iter(&mut query).with_entities() {
            entities.push(e);

            let (expected_pos, expected_rot, _) = expected_data[iter_count];

            *pos = expected_pos;
            *rot = expected_rot;

            iter_count += 1;
        }
        assert_eq!(iter_count, expected_data.len());

        let other_arch = Archetype::with_components::<(f32, u64)>();

        store.create_entity_with_archetype(&other_arch);
        store.create_entity_with_archetype(&other_arch);

        let mut query = ComponentReadOnlyQuery::<(Position, Rotation)>::new();
        let mut iter_count = 0;
        for (e, (pos, rot)) in store.component_query_iter(&mut query).with_entities() {
            iter_count += 1;

            let e_idx = entities.iter().position(|e_check| *e_check == e).unwrap();
            let (expected_pos, expected_rot, _) = expected_data[e_idx];

            assert_eq!(expected_pos, *pos);
            assert_eq!(expected_rot, *rot);
        }

        assert_eq!(iter_count, expected_data.len());
    }

    #[test]
    fn query_will_not_iterate_destroyed_entities() {
        const INIT_ENTITIES_COUNT: usize = 10;
        let mut store = Store::new();
        let arch = Archetype::with_components::<usize>();

        let entities_to_destroy = [0, 5, 9];
        let mut expected_to_iterate = Vec::new();

        for i in 0..INIT_ENTITIES_COUNT {
            let e = store.create_entity_with_archetype(&arch);
            let v = store.get_components_refs_mut::<usize>(e).unwrap();
            *v = i;

            if entities_to_destroy.contains(&i) {
                store.destroy_entity(e);
            } else {
                expected_to_iterate.push(e)
            }
        }

        let mut iterated_entities = Vec::new();
        let query = &mut ComponentReadOnlyQuery::<usize>::new();
        for (e, v) in store.component_query_iter(query).with_entities()
        {
            iterated_entities.push(e);

            assert!(entities_to_destroy.contains(v) == false);
        }

        assert!(expected_to_iterate
            .iter()
            .all(|e| iterated_entities.contains(e)));
    }
}
