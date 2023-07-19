#[macro_export]
macro_rules! component_types {
    ($($T:ident), *) => {
        [$(crate::ComponentType::of::<$T>()),*]
    };
}

#[macro_export]
macro_rules! type_ids {
    ($($T:ident), *) => {
        [$(std::any::TypeId::of::<$T>()),*]
    };
}
