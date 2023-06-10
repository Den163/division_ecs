use division_ecs::Registry;

#[test]
fn create_not_panics() {
    Registry::new();
}

#[test]
fn create_new_entity_returns_with_new_id_and_first_version() {
    let mut reg = Registry::new();
    let e1 = reg.create_entity();
    let e2 = reg.create_entity();

    assert_eq!(e1.id(), 0);
    assert_eq!(e1.version(), 1);
    assert_eq!(e2.id(), 1);
    assert_eq!(e2.version(), 1);
}

#[test]
fn create_new_entity_oversized_will_increase_capacity() {
    let mut reg = Registry::with_capacity(1);
    reg.create_entity();

    assert_eq!(reg.entities_capacity(), 1);

    reg.create_entity();

    assert_ne!(reg.entities_capacity(), 1);
}

#[test]
#[should_panic]
fn destroy_entity_when_already_destroyed_panics() {
    let mut reg = Registry::new();
    let e1 = reg.create_entity();

    reg.destroy_entity(e1);
    reg.create_entity();

    reg.destroy_entity(e1)
}

#[test]
#[should_panic]
fn destroy_entity_with_invalid_id_should_panic() {
    let mut reg = Registry::with_capacity(1);
    let mut reg2 = Registry::with_capacity(2);

    let entity_with_invalid_id = reg2.create_entity();

    reg.destroy_entity(entity_with_invalid_id);
}

#[test]
fn destroy_entity_will_increase_version_for_entity_with_same_id() {
    let mut reg = Registry::new();
    let e1 = reg.create_entity();
    reg.destroy_entity(e1);
    let e1_1 = reg.create_entity();

    assert_eq!(e1.id(), e1_1.id());
    assert_eq!(e1_1.version(), 2);
    assert_ne!(e1.version(), e1_1.version());
}

#[test]
fn is_alive_true_after_creation_false_after_destruction() {
    let mut reg = Registry::new();

    let e = reg.create_entity();
    assert!(reg.is_alive(e));

    reg.destroy_entity(e);
    assert!(reg.is_alive(e) == false);
}

#[test]
fn destroy_will_not_affect_other_entities() {
    let mut reg = Registry::new();
    let e = reg.create_entity();
    assert!(reg.is_alive(e));

    let entity_to_check = reg.create_entity();
    assert!(reg.is_alive(e));
    assert!(reg.is_alive(entity_to_check));

    reg.destroy_entity(e);
    assert!(reg.is_alive(e) == false);
    assert!(reg.is_alive(entity_to_check));
}
