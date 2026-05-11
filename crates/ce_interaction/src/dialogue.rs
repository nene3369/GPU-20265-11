//! Dialogue system — NPC conversation state and tone selection.
//!
//! The NPC's response tone is determined by its consciousness state
//! (defense strain, merit, vedana, awareness) combined with the
//! accumulated trust from the player relationship.

use ce_ai::Consciousness;

/// State of a dialogue with an NPC.
#[derive(Debug, Clone, Default)]
pub struct DialogueState {
    pub is_active: bool,
    pub current_topic: Option<String>,
    pub turn_count: u32,
    pub tone: ResponseTone,
    /// Trust level accumulated over interactions (0.0-1.0).
    pub trust_level: f32,
}

/// How the NPC responds (determined by consciousness state).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ResponseTone {
    #[default]
    Neutral,
    /// High maitri, low defense_strain.
    Warm,
    /// High defense_strain.
    Cautious,
    /// Low PE, high merit.
    Joyful,
    /// High vedana, wound_proximity.
    Sorrowful,
    /// High awareness, strong vow_weights.
    Assertive,
    /// Being asked about hearsay knowledge.
    Reluctant,
}

impl ResponseTone {
    /// Determine tone from NPC consciousness state.
    pub fn from_consciousness(c: &Consciousness, trust: f32) -> Self {
        if c.defense_strain > 0.8 && trust < 0.3 {
            return ResponseTone::Cautious;
        }
        if c.merit > 2.0 && c.prediction_error < 0.2 {
            return ResponseTone::Joyful;
        }
        if c.vedana > 0.7 {
            return ResponseTone::Sorrowful;
        }
        if c.awareness > 0.8 {
            return ResponseTone::Assertive;
        }
        if trust > 0.5 {
            return ResponseTone::Warm;
        }
        ResponseTone::Neutral
    }
}

/// Events emitted during dialogue.
#[derive(Debug, Clone)]
pub struct DialogueEvent {
    pub npc_entity: ce_core::Entity,
    pub event_type: DialogueEventType,
    pub text: Option<String>,
}

/// Types of dialogue events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogueEventType {
    Started,
    PlayerSpoke,
    NpcResponded,
    TopicChanged,
    Ended,
    TrustIncreased,
    TrustDecreased,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_defense_low_trust_yields_cautious() {
        let mut c = Consciousness::default();
        c.defense_strain = 0.9;
        let tone = ResponseTone::from_consciousness(&c, 0.1);
        assert_eq!(tone, ResponseTone::Cautious);
    }

    #[test]
    fn high_merit_low_pe_yields_joyful() {
        let mut c = Consciousness::default();
        c.merit = 3.0;
        c.prediction_error = 0.1;
        c.defense_strain = 0.0; // ensure we don't hit Cautious
        let tone = ResponseTone::from_consciousness(&c, 0.1);
        assert_eq!(tone, ResponseTone::Joyful);
    }

    #[test]
    fn high_vedana_yields_sorrowful() {
        let mut c = Consciousness::default();
        c.vedana = 0.9;
        c.defense_strain = 0.0;
        let tone = ResponseTone::from_consciousness(&c, 0.1);
        assert_eq!(tone, ResponseTone::Sorrowful);
    }

    #[test]
    fn high_awareness_yields_assertive() {
        let mut c = Consciousness::default();
        c.awareness = 0.9;
        c.defense_strain = 0.0;
        c.vedana = 0.0;
        let tone = ResponseTone::from_consciousness(&c, 0.1);
        assert_eq!(tone, ResponseTone::Assertive);
    }

    #[test]
    fn high_trust_yields_warm() {
        let c = Consciousness::default(); // awareness=0.5, vedana=0, defense=0
        let tone = ResponseTone::from_consciousness(&c, 0.7);
        assert_eq!(tone, ResponseTone::Warm);
    }

    #[test]
    fn default_yields_neutral() {
        let c = Consciousness::default();
        let tone = ResponseTone::from_consciousness(&c, 0.2);
        assert_eq!(tone, ResponseTone::Neutral);
    }

    #[test]
    fn dialogue_state_default() {
        let ds = DialogueState::default();
        assert!(!ds.is_active);
        assert!(ds.current_topic.is_none());
        assert_eq!(ds.turn_count, 0);
        assert_eq!(ds.tone, ResponseTone::Neutral);
        assert_eq!(ds.trust_level, 0.0);
    }
}
