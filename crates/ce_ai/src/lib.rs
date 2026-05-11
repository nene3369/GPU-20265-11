//! # ce_ai — ChemEngine AI / NPC Consciousness
//!
//! Implements NPC consciousness inspired by Buddhist philosophy
//! (SMA28 Shion architecture). Provides the consciousness kernel,
//! three poisons model, emotional state, four immeasurables reward
//! function, and contact-tier knowledge provenance.

pub mod consciousness;
pub mod contact_tier;
pub mod emotion;
pub mod four_immeasurables;
pub mod three_poisons;

pub use consciousness::Consciousness;
pub use contact_tier::{ContactTier, Knowledge, KnowledgeBase};
pub use emotion::{EmotionalState, EmotionalWeight};
pub use four_immeasurables::FourImmeasurables;
pub use three_poisons::{PoisonType, ThreePoisons};

use ce_app::{App, Plugin};

/// Plugin that activates the NPC consciousness subsystem.
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        let _ = app; // currently no systems/resources to register
        log::info!("AiPlugin loaded — consciousness kernel active");
    }
}
