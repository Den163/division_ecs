#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::resource_store::ResourceStore;
    const EXPECTED_STRING: &str = "String to check";

    thread_local! {
        static DROP_COUNTS: RefCell<usize> = RefCell::new(0);
    }

    struct DropCounter;

    impl DropCounter {
        fn reset() {
            DROP_COUNTS.set(0);
        }

        fn drop_counts_eq(drop_counts: usize) -> bool {
            unsafe { DROP_COUNTS.with(|v| *v.as_ptr() == drop_counts) }
        }
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            DROP_COUNTS.set(DROP_COUNTS.take() + 1);
        }
    }

    #[test]
    fn resources_store_index_as_expected() {
        let mut store = ResourceStore::new();
        let e = store.create(String::from(EXPECTED_STRING));

        let val = &store[e];
        assert_eq!(val, EXPECTED_STRING);
    }

    #[test]
    fn resources_store_index_mut_as_expected() {
        const NEW_STRING: &str = "NewString";

        let mut store = ResourceStore::new();
        let e = store.create(String::from(EXPECTED_STRING));

        let val = &mut store[e];
        *val = String::from(NEW_STRING);

        assert_eq!(&store[e], NEW_STRING);
    }

    #[test]
    fn resources_store_release_drops_resource() {
        DropCounter::reset();

        let mut store = ResourceStore::new();
        let e = store.create(DropCounter);
        {
            store.release(e);
        }

        assert!(DropCounter::drop_counts_eq(1));
    }

    #[test]
    fn resources_store_drops_as_expected() {
        DropCounter::reset();

        {
            let mut store = ResourceStore::new();
            store.create(DropCounter);
        }

        assert!(DropCounter::drop_counts_eq(1));
    }

    #[test]
    fn resources_store_drops_many_times_as_expected() {
        DropCounter::reset();
        const EXPECTED_DROPS: usize = 100;

        {
            let mut store = ResourceStore::with_capacity(EXPECTED_DROPS);

            for _ in 0..EXPECTED_DROPS {
                store.create(DropCounter);
            }
        }

        assert!(DropCounter::drop_counts_eq(EXPECTED_DROPS))
    }
}
