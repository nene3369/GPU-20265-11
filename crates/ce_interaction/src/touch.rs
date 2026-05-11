//! Touch / petting system — NPC responses to physical contact.
//!
//! Determines how an NPC reacts to being touched based on the touch
//! zone, the NPC's consciousness state, and the accumulated trust level.
//! Reactions feed back into the consciousness model (defense strain,
//! merit, prediction error).

use ce_ai::Consciousness;

/// Where on the NPC the player is touching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TouchZone {
    Head,
    Face,
    Shoulder,
    Hand,
    Back,
    Arm,
    #[default]
    None,
}

/// Current touch state for an NPC.
#[derive(Debug, Clone, Default)]
pub struct TouchState {
    pub zone: TouchZone,
    pub is_being_touched: bool,
    /// Duration of current touch in seconds.
    pub touch_duration: f32,
    /// Pressure of touch (0.0-1.0).
    pub touch_pressure: f32,
    /// Movement speed of touch.
    pub stroke_speed: f32,
    /// How many times touched today.
    pub touch_count_today: u32,
}

/// NPC response to being touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PettingResponse {
    /// Relaxes, purrs, leans in.
    Enjoy,
    /// Accepts but doesn't react much.
    Tolerate,
    /// Blushes, looks away, but doesn't reject.
    Shy,
    /// Startled but not negative.
    Surprise,
    /// Pulls away, defense_strain increases.
    Reject,
    /// Deep trust moment, merit increases for both.
    Trust,
}

impl PettingResponse {
    /// Determine response based on NPC consciousness, touch zone, and trust.
    pub fn determine(
        consciousness: &Consciousness,
        zone: TouchZone,
        trust: f32,
        touch_duration: f32,
    ) -> Self {
        let comfort = trust * (1.0 - consciousness.defense_strain as f32);

        match zone {
            TouchZone::Head => {
                if comfort > 0.6 {
                    PettingResponse::Enjoy
                } else if comfort > 0.3 {
                    PettingResponse::Shy
                } else {
                    PettingResponse::Reject
                }
            }
            TouchZone::Face => {
                if comfort > 0.8 {
                    PettingResponse::Trust
                } else if comfort > 0.5 {
                    PettingResponse::Shy
                } else {
                    PettingResponse::Reject
                }
            }
            TouchZone::Shoulder | TouchZone::Arm => {
                if comfort > 0.4 {
                    PettingResponse::Tolerate
                } else {
                    PettingResponse::Surprise
                }
            }
            TouchZone::Hand => {
                if comfort > 0.5 {
                    PettingResponse::Enjoy
                } else if touch_duration < 0.5 {
                    PettingResponse::Surprise
                } else {
                    PettingResponse::Shy
                }
            }
            TouchZone::Back => {
                if comfort > 0.7 {
                    PettingResponse::Enjoy
                } else {
                    PettingResponse::Tolerate
                }
            }
            TouchZone::None => PettingResponse::Tolerate,
        }
    }

    /// How this response affects the NPC's consciousness.
    ///
    /// Returns `(defense_strain_delta, merit_delta, pe_delta)`.
    pub fn consciousness_effect(&self) -> (f64, f64, f64) {
        match self {
            PettingResponse::Enjoy => (-0.05, 0.02, -0.01),
            PettingResponse::Tolerate => (0.0, 0.005, 0.0),
            PettingResponse::Shy => (0.02, 0.01, 0.02),
            PettingResponse::Surprise => (0.05, 0.0, 0.05),
            PettingResponse::Reject => (0.1, -0.01, 0.1),
            PettingResponse::Trust => (-0.1, 0.05, -0.05),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_trust_head_yields_enjoy() {
        let c = Consciousness::default(); // defense_strain = 0.0
        let resp = PettingResponse::determine(&c, TouchZone::Head, 0.8, 1.0);
        assert_eq!(resp, PettingResponse::Enjoy);
    }

    #[test]
    fn low_trust_face_yields_reject() {
        let c = Consciousness::default();
        let resp = PettingResponse::determine(&c, TouchZone::Face, 0.2, 1.0);
        assert_eq!(resp, PettingResponse::Reject);
    }

    #[test]
    fn very_high_trust_face_yields_trust() {
        let c = Consciousness::default(); // defense_strain = 0.0
        let resp = PettingResponse::determine(&c, TouchZone::Face, 0.9, 1.0);
        // comfort = 0.9 * (1.0 - 0.0) = 0.9 > 0.8 => Trust
        assert_eq!(resp, PettingResponse::Trust);
    }

    #[test]
    fn high_defense_reduces_comfort() {
        let mut c = Consciousness::default();
        c.defense_strain = 0.8;
        // comfort = 0.8 * (1.0 - 0.8) = 0.16
        let resp = PettingResponse::determine(&c, TouchZone::Head, 0.8, 1.0);
        assert_eq!(resp, PettingResponse::Reject);
    }

    #[test]
    fn shoulder_moderate_comfort_yields_tolerate() {
        let c = Consciousness::default();
        let resp = PettingResponse::determine(&c, TouchZone::Shoulder, 0.6, 1.0);
        assert_eq!(resp, PettingResponse::Tolerate);
    }

    #[test]
    fn shoulder_low_comfort_yields_surprise() {
        let mut c = Consciousness::default();
        c.defense_strain = 0.8;
        let resp = PettingResponse::determine(&c, TouchZone::Shoulder, 0.3, 1.0);
        // comfort = 0.3 * 0.2 = 0.06
        assert_eq!(resp, PettingResponse::Surprise);
    }

    #[test]
    fn hand_brief_touch_low_trust_yields_surprise() {
        let c = Consciousness::default();
        let resp = PettingResponse::determine(&c, TouchZone::Hand, 0.3, 0.3);
        // comfort = 0.3 * 1.0 = 0.3, not > 0.5, touch_duration < 0.5
        assert_eq!(resp, PettingResponse::Surprise);
    }

    #[test]
    fn hand_long_touch_low_trust_yields_shy() {
        let c = Consciousness::default();
        let resp = PettingResponse::determine(&c, TouchZone::Hand, 0.3, 1.0);
        // comfort = 0.3, not > 0.5, touch_duration >= 0.5
        assert_eq!(resp, PettingResponse::Shy);
    }

    #[test]
    fn none_zone_yields_tolerate() {
        let c = Consciousness::default();
        let resp = PettingResponse::determine(&c, TouchZone::None, 0.0, 0.0);
        assert_eq!(resp, PettingResponse::Tolerate);
    }

    #[test]
    fn enjoy_effect_reduces_defense() {
        let (ds, merit, pe) = PettingResponse::Enjoy.consciousness_effect();
        assert!(ds < 0.0, "Enjoy should reduce defense strain");
        assert!(merit > 0.0, "Enjoy should increase merit");
        assert!(pe < 0.0, "Enjoy should reduce prediction error");
    }

    #[test]
    fn reject_effect_increases_defense() {
        let (ds, merit, pe) = PettingResponse::Reject.consciousness_effect();
        assert!(ds > 0.0, "Reject should increase defense strain");
        assert!(merit < 0.0, "Reject should decrease merit");
        assert!(pe > 0.0, "Reject should increase prediction error");
    }

    #[test]
    fn trust_effect_is_strongly_positive() {
        let (ds, merit, pe) = PettingResponse::Trust.consciousness_effect();
        assert!(ds < 0.0, "Trust should reduce defense strain");
        assert!(merit > 0.0, "Trust should increase merit");
        assert!(pe < 0.0, "Trust should reduce prediction error");
        // Trust should have stronger effects than Enjoy
        let (eds, emerit, _) = PettingResponse::Enjoy.consciousness_effect();
        assert!(ds < eds, "Trust defense reduction should exceed Enjoy");
        assert!(merit > emerit, "Trust merit gain should exceed Enjoy");
    }

    #[test]
    fn tolerate_effect_is_neutral_ish() {
        let (ds, merit, pe) = PettingResponse::Tolerate.consciousness_effect();
        assert_eq!(ds, 0.0);
        assert!(merit > 0.0, "Tolerate still gains a tiny bit of merit");
        assert_eq!(pe, 0.0);
    }
}
