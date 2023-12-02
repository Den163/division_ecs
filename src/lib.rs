pub mod component_query;
pub mod entity_component_query;

mod archetype;

mod archetype_builder;

mod archetype_data_page;
mod archetype_data_page_view;
mod archetypes_container;
mod bitvec_utils;
mod component;
mod component_query_access;
mod component_type;
mod entities_container;
mod entity;
mod entity_in_archetype;
mod mem_utils;
mod resource_store;
mod store;
mod tests;
mod tuple;

pub mod macros;

pub(crate) use archetype::tuple_into_archetype_impl;
pub(crate) use archetype_builder::archetype_builder_tuple_impl;

pub use archetype::Archetype;
pub use archetype_builder::ArchetypeBuilder;
pub use component::Component;
pub use component_query::{
    ComponentQuery, ComponentReadOnlyQuery, ComponentReadWriteQuery, ComponentWriteQuery,
};
pub use component_query_access::{ReadWriteAccess, ReadonlyAccess, WriteAccess};
pub use component_type::ComponentType;
pub use entity_component_query::{
    EntityComponentQuery, EntityComponentQueryIter, EntityComponentReadOnlyQuery,
    EntityComponentReadWriteQuery, EntityComponentWriteQuery,
};

pub use entity::Entity;
pub use resource_store::ResourceStore;
pub use store::Store;
