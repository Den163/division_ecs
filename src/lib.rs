extern crate core;

mod archetype;
mod archetype_builder;
mod component_type;
mod entities_container;
mod entity;
mod mem_utils;
mod registry;

pub mod archetype_data_layout;
pub mod archetype_data_page;

pub use archetype_builder::ArchetypeBuilder;
pub use entity::Entity;
pub use registry::Registry;

use entities_container::EntitiesContainer;
