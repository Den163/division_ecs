#[cfg(test)]
mod tests {
    use crate::{Store, Tag};

    #[derive(Tag)]
    struct TestTag;
    
    #[derive(Tag)]
    struct OtherTestTag;

    #[test]
    fn add_remove_tag_as_expected() {
        let mut store = Store::new();

        let e = store.create_entity();
        store.add_tag::<TestTag>(e);

        assert!(store.has_tag::<TestTag>(e));

        store.remove_tag::<TestTag>(e);

        assert!(store.has_tag::<TestTag>(e) == false);
    }

    #[test]
    fn add_one_tag_doesnt_affect_on_other() {
        let mut store = Store::new();

        let e = store.create_entity();
        store.add_tag::<TestTag>(e);
        store.add_tag::<OtherTestTag>(e);

        assert!(store.has_tag::<TestTag>(e));
        assert!(store.has_tag::<OtherTestTag>(e));

        store.remove_tag::<OtherTestTag>(e);

        assert!(store.has_tag::<TestTag>(e));
        assert!(store.has_tag::<OtherTestTag>(e) == false);

        store.remove_tag::<TestTag>(e);

        assert!(store.has_tag::<TestTag>(e) == false);
        assert!(store.has_tag::<OtherTestTag>(e) == false);
    }
}