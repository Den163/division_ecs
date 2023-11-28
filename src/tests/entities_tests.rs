#[cfg(test)]
mod tests {
    use crate::{Archetype, ArchetypeBuilder, Component, Store};

    impl Component for u32 {}

    #[test]
    fn create_not_panics() {
        Store::new();
    }

    #[test]
    fn create_new_entity_returns_with_new_id_and_first_version() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::new();
        let e1 = reg.create_entity(&arch_stub);
        let e2 = reg.create_entity(&arch_stub);

        assert_eq!(e1.id(), 0);
        assert_eq!(e1.version(), 1);
        assert_eq!(e2.id(), 1);
        assert_eq!(e2.version(), 1);
    }

    #[test]
    fn create_new_entity_oversized_will_increase_capacity() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::with_capacity(1);
        reg.create_entity(&arch_stub);

        assert_eq!(reg.entities_capacity(), 1);

        reg.create_entity(&arch_stub);

        assert_ne!(reg.entities_capacity(), 1);
    }

    #[test]
    #[should_panic]
    fn destroy_entity_when_already_destroyed_panics() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::new();
        let e1 = reg.create_entity(&arch_stub);

        reg.destroy_entity(e1);
        reg.create_entity(&arch_stub);

        reg.destroy_entity(e1)
    }

    #[test]
    #[should_panic]
    fn destroy_entity_with_invalid_id_should_panic() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::with_capacity(1);
        let mut reg2 = Store::with_capacity(2);

        let entity_with_invalid_id = reg2.create_entity(&arch_stub);

        reg.destroy_entity(entity_with_invalid_id);
    }

    #[test]
    fn destroy_entity_will_increase_version_for_entity_with_same_id() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::new();
        let e1 = reg.create_entity(&arch_stub);
        reg.destroy_entity(e1);
        let e1_1 = reg.create_entity(&arch_stub);

        assert_eq!(e1.id(), e1_1.id());
        assert_eq!(e1_1.version(), 2);
        assert_ne!(e1.version(), e1_1.version());
    }

    #[test]
    fn is_alive_true_after_creation_false_after_destruction() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::new();

        let e = reg.create_entity(&arch_stub);
        assert!(reg.is_alive(e));

        reg.destroy_entity(e);
        assert!(reg.is_alive(e) == false);
    }

    #[test]
    fn destroy_will_not_affect_other_entities() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::new();
        let e = reg.create_entity(&arch_stub);
        assert!(reg.is_alive(e));

        let entity_to_check = reg.create_entity(&arch_stub);
        assert!(reg.is_alive(e));
        assert!(reg.is_alive(entity_to_check));

        reg.destroy_entity(e);
        assert!(reg.is_alive(e) == false);
        assert!(reg.is_alive(entity_to_check));
    }

    #[test]
    fn increase_capacity_will_not_affect_other_entities() {
        let arch_stub = create_archetype_stub();
        let mut reg = Store::new();
        let mut entities = Vec::new();

        for _ in 0..33 {
            let e = reg.create_entity(&arch_stub);
            assert!(reg.is_alive(e));
            
            entities.push(e);
        }

        assert!(entities.into_iter().all(|e| reg.is_alive(e)));
    }

    fn create_archetype_stub() -> Archetype {
        ArchetypeBuilder::new().include_components::<u32>().build()
    }
}
