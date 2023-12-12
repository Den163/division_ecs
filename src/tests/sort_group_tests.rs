#[cfg(test)]
mod tests {
    use crate::{Entity, Store, Tag};

    #[derive(Tag)]
    struct TestGroup;

    #[test]
    fn get_first_or_last_entity_ordered_by_returns_none_when_no_order_group() {
        let store = Store::new();

        assert_eq!(store.get_first_entity_ordered_by::<TestGroup>(), None);
        assert_eq!(store.get_last_entity_ordered_by::<TestGroup>(), None);
    }

    #[test]
    fn add_entity_order_by_after_first_call_as_expected() {
        let mut store = Store::new();
        let entity = store.create_entity();

        store.add_entity_order_by::<TestGroup>(entity);

        assert_eq!(
            store.get_first_entity_ordered_by::<TestGroup>(),
            Some(entity)
        );
        assert_eq!(
            store.get_last_entity_ordered_by::<TestGroup>(),
            Some(entity)
        );
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(entity), None);
        assert_eq!(
            store.get_previous_entity_ordered_by::<TestGroup>(entity),
            None
        );
    }

    #[test]
    fn add_entity_order_by_multiple_times_order_as_expected() {
        let mut store = Store::new();
        let e0 = store.create_entity();
        let e1 = store.create_entity();

        store.add_entity_order_by::<TestGroup>(e0);
        store.add_entity_order_by::<TestGroup>(e1);

        assert_two_entities_add_as_expected(&store, e0, e1);
    }

    #[test]
    fn add_entity_order_by_multiple_times_after_resizing_order_as_expected() {
        let mut store = Store::with_capacity(1);

        let e0 = store.create_entity();
        let e1 = store.create_entity();

        store.add_entity_order_by::<TestGroup>(e0);
        store.add_entity_order_by::<TestGroup>(e1);

        assert_two_entities_add_as_expected(&store, e0, e1);
    }

    fn assert_two_entities_add_as_expected(store: &Store, e0: Entity, e1: Entity) {
        assert_eq!(store.get_first_entity_ordered_by::<TestGroup>(), Some(e0));
        assert_eq!(store.get_last_entity_ordered_by::<TestGroup>(), Some(e1));

        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e0), Some(e1));
        assert_eq!(
            store.get_previous_entity_ordered_by::<TestGroup>(e1),
            Some(e0)
        );

        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e1), None);
        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e0), None);
    }

    #[test]
    fn remove_entity_order_by_from_when_no_group_do_nothing() {
        let mut store = Store::new();
        let e0 = store.create_entity();

        store.remove_entity_order_by::<TestGroup>(e0);

        assert!(store.is_alive(e0));
    }

    #[test]
    fn remove_entity_order_by_after_first_add_as_expected() {
        let mut store = Store::new();
        let entity = store.create_entity();

        store.add_entity_order_by::<TestGroup>(entity);
        store.remove_entity_order_by::<TestGroup>(entity);

        assert_eq!(store.get_first_entity_ordered_by::<TestGroup>(), None);
        assert_eq!(store.get_last_entity_ordered_by::<TestGroup>(), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(entity), None);
        assert_eq!(
            store.get_previous_entity_ordered_by::<TestGroup>(entity),
            None
        );
    }

    #[test]
    fn remove_entity_order_by_multiple_times_as_expected() {
        let mut store = Store::with_capacity(3);

        let e0 = store.create_entity();
        let e1 = store.create_entity();
        let e2 = store.create_entity();

        store.add_entity_order_by::<TestGroup>(e0);
        store.add_entity_order_by::<TestGroup>(e1);
        store.add_entity_order_by::<TestGroup>(e2);

        store.remove_entity_order_by::<TestGroup>(e1);

        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e1), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e1), None);

        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e0), Some(e2));
        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e2), Some(e0));

        assert_eq!(store.get_first_entity_ordered_by::<TestGroup>(), Some(e0));
        assert_eq!(store.get_last_entity_ordered_by::<TestGroup>(), Some(e2));

        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e2), None);
        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e0), None);

        store.remove_entity_order_by::<TestGroup>(e0);

        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e0), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e0), None);

        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e1), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e1), None);
        
        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e2), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e2), None);

        assert_eq!(store.get_first_entity_ordered_by::<TestGroup>(), Some(e2));
        assert_eq!(store.get_last_entity_ordered_by::<TestGroup>(), Some(e2));

        store.remove_entity_order_by::<TestGroup>(e2);

        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e0), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e0), None);

        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e1), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e1), None);
        
        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e2), None);
        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e2), None);

        assert_eq!(store.get_first_entity_ordered_by::<TestGroup>(), None);
        assert_eq!(store.get_last_entity_ordered_by::<TestGroup>(), None);
    }

    #[test]
    fn after_entity_destroy_order_is_removed_too() {
        let mut store = Store::new();

        let e0 = store.create_entity();
        let e1 = store.create_entity();
        let e2 = store.create_entity();

        store.add_entity_order_by::<TestGroup>(e0);
        store.add_entity_order_by::<TestGroup>(e1);
        store.add_entity_order_by::<TestGroup>(e2);

        store.destroy_entity(e1);

        assert_eq!(store.get_next_entity_ordered_by::<TestGroup>(e0), Some(e2));
        assert_eq!(store.get_previous_entity_ordered_by::<TestGroup>(e2), Some(e0));
    }
}
