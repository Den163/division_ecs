#[cfg(test)]
mod tests {
    use division_ecs::{Entity, Registry};

    #[test]
    fn create_not_panics() {
        Registry::new();
    }

    #[test]
    fn create_new_entity_returns_with_new_id_and_first_version() {
        let mut reg = Registry::new();
        let e1 = reg.create_entity();
        let e2 = reg.create_entity();

        assert_eq!(e1, Entity { id: 0, version: 1 });
        assert_eq!(e2, Entity { id: 1, version: 1 });
        assert_ne!(e1.id, e2.id);
    }

    #[test]
    fn destroy_entity_will_increase_version_for_entity_with_same_id() {
        let mut reg = Registry::new();
        let e1 = reg.create_entity();
        reg.destroy_entity(e1);
        let e1_1 = reg.create_entity();

        assert_eq!(e1.id, e1_1.id);
        assert_eq!(e1_1.version, 2);
        assert_ne!(e1.version, e1_1.version);
    }

    #[test]
    fn is_alive_as_expected() {
        let mut reg = Registry::new();
        let e = reg.create_entity();

        assert!(reg.is_alive(e));

        reg.destroy_entity(e);
        assert!(!reg.is_alive(e));
    }
}
