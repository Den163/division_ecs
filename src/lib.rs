mod archetype;
mod archetype_builder;
mod archetype_data_page_entities_iterator;
mod archetype_data_page_view;
mod archetypes_container;
mod component_type;
mod entity;
mod entity_in_archetype;
mod entity_in_archetype_data_page;
mod entities_container;
mod mem_utils;
mod registry;
mod registry_quries;
mod tests;

pub mod archetype_data_layout;
pub mod archetype_data_page;
pub mod macros;

pub use archetype::Archetype;
pub use archetype_builder::ArchetypeBuilder;
pub use component_type::ComponentType;
pub use entity::Entity;
pub use registry::Registry;

use entities_container::EntitiesContainer;