mod archetype;

mod archetype_builder;

mod archetype_data_page;
mod archetype_data_page_view;
mod archetype_layout;
mod archetypes_container;
mod bitvec_utils;
mod component;
mod component_tuple;
mod component_type;
mod entities_container;
mod entity;
mod entity_in_archetype;
mod mem_utils;
mod resource_store;
mod store;
mod tests;

pub mod macros;
pub mod query;

pub use archetype::Archetype;
pub use archetype_builder::ArchetypeBuilder;
pub use component::Component;
pub use component_type::ComponentType;

pub use entity::Entity;
pub use resource_store::ResourceStore;
pub use store::Store;

pub use query::component::{
    ComponentQuery, ComponentReadOnlyQuery, ComponentReadWriteQuery, ComponentWriteQuery,
};
pub use query::access::{ReadWriteAccess, ReadonlyAccess, WriteAccess};
pub use query::entity_component::{
    EntityComponentQuery, EntityComponentQueryIter, EntityComponentReadOnlyQuery,
    EntityComponentReadWriteQuery, EntityComponentWriteQuery,
};
