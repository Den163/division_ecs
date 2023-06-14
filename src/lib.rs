extern crate core;

mod archetype;
mod archetype_builder;
mod archetypes_container;
mod component_type;
mod entities_container;
mod entity_in_archetype;
mod entity;
mod mem_utils;
mod registry;

pub mod archetype_data_layout;
pub mod archetype_data_page;

pub use archetype::Archetype;
pub use archetype_builder::ArchetypeBuilder;
pub use entity::Entity;
pub use registry::Registry;

use entities_container::EntitiesContainer;
