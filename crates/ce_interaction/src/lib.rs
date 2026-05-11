//! # ce_interaction — ChemEngine Player-NPC Interaction System
//!
//! The CORE differentiator of ChemEngine: makes NPCs "feel" the player and
//! respond with genuine consciousness-driven behaviour.
//!
//! Bridges XR sensor data (face tracking, eye tracking, voice, body) into
//! the NPC consciousness model, driving dialogue, touch responses, memory
//! formation, and proxemics-aware social behaviour.

pub mod dialogue;
pub mod memory;
pub mod perception;
pub mod proximity;
pub mod touch;

pub use dialogue::{DialogueEvent, DialogueState, ResponseTone};
pub use memory::{MemoryEntry, NpcMemory, Relationship};
pub use perception::{PerceivedEmotion, PerceivedIntent, PlayerPerception};
pub use proximity::{ProximityState, SocialDistance};
pub use touch::{PettingResponse, TouchState, TouchZone};

use ce_app::{App, Plugin};
use ce_ecs::CoreStage;

/// Plugin that activates the player-NPC interaction subsystem.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(CoreStage::PreUpdate, perception::update_player_perception);
        app.add_system(CoreStage::Update, proximity::update_proximity);
        log::info!("InteractionPlugin loaded — emotion loop active");
    }
}
