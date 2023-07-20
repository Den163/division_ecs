mod archetype;
mod archetype_builder;
mod archetype_data_page_view;
mod archetypes_container;
mod component_type;
mod entity;
mod entity_in_archetype;
mod entities_container;
mod mem_utils;
mod registry;
mod components_read_query;
mod tests;
mod tuple;

pub mod archetype_data_layout;
pub mod archetype_data_page;
pub mod macros;

pub use archetype::Archetype;
pub use archetype_builder::ArchetypeBuilder;
pub use component_type::ComponentType;
pub use components_read_query::ReadQuery;
pub use components_read_query::QueryIterator;
pub use entity::Entity;
pub use registry::Registry;

use entities_container::EntitiesContainer;