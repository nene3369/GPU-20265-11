//! # ce_ecs — ChemEngine Entity Component System
//!
//! An archetype-based ECS where entities sharing the same component set are
//! stored together in cache-friendly SoA (Struct of Arrays) tables.
//!
//! ## Quick Start
//!
//! ```rust
//! use ce_ecs::prelude::*;
//!
//! let mut world = World::new();
//! let entity = world.spawn();
//! world.insert_component(entity, 42u32);
//! assert_eq!(world.get_component::<u32>(entity), Some(&42));
//! ```

pub mod archetype;
pub mod component;
pub mod event;
pub mod resource;
pub mod schedule;
pub mod world;

// Re-exports for ergonomic use.
pub use archetype::{Archetype, ArchetypeId};
pub use component::{Component, ComponentColumn};
pub use event::{EventReader, EventWriter, Events};
pub use resource::{Resource, Resources};
pub use schedule::{CoreStage, Schedule};
pub use world::World;

// Re-export Entity from ce_core for convenience.
pub use ce_core::Entity;

/// Prelude module for glob imports.
///
/// ```rust
/// use ce_ecs::prelude::*;
/// ```
pub mod prelude {
    pub use crate::component::Component;
    pub use crate::event::{EventReader, EventWriter, Events};
    pub use crate::resource::Resource;
    pub use crate::schedule::{CoreStage, Schedule};
    pub use crate::world::World;
    pub use ce_core::Entity;
}
