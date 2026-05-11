//! # ce_app — ChemEngine Application Framework
//!
//! Provides the [`App`] builder, [`Plugin`] trait, and frame-timing
//! resources ([`Time`], [`FixedTime`]) that form the backbone of every
//! ChemEngine application.
//!
//! `App` owns the ECS [`World`](ce_ecs::World) and
//! [`Schedule`](ce_ecs::Schedule), wires up plugins, and drives the
//! per-frame update loop.  It intentionally does **not** contain a
//! windowing event loop — that responsibility belongs to `ce_window`.

pub mod app;
pub mod plugin;
pub mod time;

// ── Public re-exports ──────────────────────────────────────────────
pub use app::App;
pub use plugin::Plugin;
pub use time::{FixedTime, Time};
