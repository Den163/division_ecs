mod archetype;

mod archetype_builder;

mod archetype_data_layout;
mod archetype_data_page;
mod archetype_data_page_view;
mod archetypes_container;
mod bitvec_utils;
mod component;
mod component_type;
mod components_query;
mod components_query_access;
mod entities_container;
mod entity;
mod entity_in_archetype;
mod mem_utils;
mod store;
mod tests;
mod tuple;

pub mod macros;

pub(crate) use archetype_builder::archetype_builder_tuple_impl;

pub use archetype::Archetype;
pub use archetype_builder::ArchetypeBuilder;
pub use component::Component;
pub use component_type::ComponentType;
pub use components_query::{
    ComponentsQuery, ComponentsReadOnlyQuery, ComponentsReadWriteQuery,
    ComponentsWriteQuery,
};
pub use entity::Entity;
pub use store::Store;
