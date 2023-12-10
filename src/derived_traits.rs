pub use division_ecs_attributes::{Component, Tag};

pub trait Component: Clone + Copy + Sized {}

pub trait Tag {}