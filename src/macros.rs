#[macro_export]
macro_rules! component_types {
    ($($T:ident), *) => {
        [$(ComponentType::of::<$T>()),*]
    };
}

#[macro_export]
macro_rules! type_ids {
    ($($T:ident), *) => {
        [$(TypeId::of::<T>()),*]
    };
}
