//! XR-to-AI bridge — the emotion loop core.
//!
//! Converts XR sensor data (face tracking, eye tracking, voice, body)
//! into NPC-understandable perception that feeds into the consciousness
//! kernel each frame.

use ce_math::Vec3;

/// What the NPC perceives about the player, updated each frame.
#[derive(Debug, Clone, Default)]
pub struct PlayerPerception {
    /// Perceived emotional state of the player (from face tracking).
    pub emotion: PerceivedEmotion,
    /// Perceived intent (from voice + body language).
    pub intent: PerceivedIntent,
    /// Where the player is looking (gaze direction in world space).
    pub gaze_target: Option<Vec3>,
    /// Is the player looking at this NPC?
    pub is_being_looked_at: bool,
    /// Distance to the player.
    pub distance: f32,
    /// How long the player has been nearby (seconds).
    pub nearby_duration: f32,
    /// Is the player speaking?
    pub is_speaking: bool,
    /// Last recognized speech text.
    pub last_speech: Option<String>,
    /// Confidence in the overall perception (0.0-1.0).
    pub confidence: f32,
}

/// Perceived emotion of the player, derived from face tracking blend shapes.
#[derive(Debug, Clone, Copy, Default)]
pub struct PerceivedEmotion {
    pub happiness: f32,
    pub sadness: f32,
    pub anger: f32,
    pub surprise: f32,
    pub fear: f32,
    pub neutral: f32,
    pub confidence: f32,
}

impl PerceivedEmotion {
    /// Create from face tracking detected emotion.
    pub fn from_face_tracking(face: &ce_xr::DetectedEmotion) -> Self {
        Self {
            happiness: face.happiness,
            sadness: face.sadness,
            anger: face.anger,
            surprise: face.surprise,
            fear: 0.0,
            neutral: face.neutral,
            confidence: face.confidence,
        }
    }

    /// Dominant perceived emotion name.
    pub fn dominant(&self) -> &'static str {
        let vals = [
            (self.happiness, "happiness"),
            (self.sadness, "sadness"),
            (self.anger, "anger"),
            (self.surprise, "surprise"),
            (self.fear, "fear"),
            (self.neutral, "neutral"),
        ];
        vals.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap()
            .1
    }

    /// Is the player showing positive emotion?
    pub fn is_positive(&self) -> bool {
        self.happiness > 0.3 || self.surprise > 0.3
    }

    /// Is the player showing negative emotion?
    pub fn is_negative(&self) -> bool {
        self.sadness > 0.3 || self.anger > 0.3 || self.fear > 0.3
    }
}

/// Perceived intent of the player.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PerceivedIntent {
    #[default]
    Neutral,
    /// Approaching calmly, smiling.
    Friendly,
    /// Looking at NPC, exploring.
    Curious,
    /// Moving quickly toward, angry expression.
    Aggressive,
    /// Moving away.
    Retreating,
    /// Reaching out to touch, gentle movement.
    Affectionate,
    /// Strong voice, pointing.
    Commanding,
    /// Sad expression, slow approach.
    Pleading,
}

impl PerceivedIntent {
    /// Infer intent from combined XR signals.
    pub fn infer(
        emotion: &PerceivedEmotion,
        distance: f32,
        is_approaching: bool,
        is_reaching: bool,
        voice_volume: f32,
    ) -> Self {
        if is_reaching && emotion.happiness > 0.3 {
            return PerceivedIntent::Affectionate;
        }
        if emotion.anger > 0.5 && is_approaching {
            return PerceivedIntent::Aggressive;
        }
        if emotion.sadness > 0.4 && distance < 2.0 {
            return PerceivedIntent::Pleading;
        }
        if voice_volume > 0.7 && emotion.anger > 0.2 {
            return PerceivedIntent::Commanding;
        }
        if !is_approaching && distance > 3.0 {
            return PerceivedIntent::Retreating;
        }
        if emotion.happiness > 0.3 && is_approaching {
            return PerceivedIntent::Friendly;
        }
        if distance < 3.0 {
            return PerceivedIntent::Curious;
        }
        PerceivedIntent::Neutral
    }
}

/// System that updates PlayerPerception from XR sensor data.
/// This is the core of the emotion loop.
pub fn update_player_perception(world: &mut ce_ecs::World) {
    // In a full implementation, this would:
    // 1. Read FaceTracking resource -> PerceivedEmotion
    // 2. Read EyeTracking resource -> gaze_target, is_being_looked_at
    // 3. Read VoiceInput resource -> is_speaking, last_speech
    // 4. Read BodyTracking resource -> distance, is_approaching, is_reaching
    // 5. Combine into PerceivedIntent
    // 6. Update PlayerPerception resource
    // 7. Feed into each NPC's Consciousness::step()
    //
    // For now this is a placeholder that will be connected in M2.
    let _ = world;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominant_returns_correct_emotion() {
        let e = PerceivedEmotion {
            happiness: 0.1,
            sadness: 0.9,
            anger: 0.2,
            surprise: 0.0,
            fear: 0.0,
            neutral: 0.1,
            confidence: 0.8,
        };
        assert_eq!(e.dominant(), "sadness");

        let e2 = PerceivedEmotion {
            happiness: 0.8,
            sadness: 0.1,
            anger: 0.0,
            surprise: 0.2,
            fear: 0.0,
            neutral: 0.1,
            confidence: 0.9,
        };
        assert_eq!(e2.dominant(), "happiness");
    }

    #[test]
    fn is_positive_detects_happy() {
        let e = PerceivedEmotion {
            happiness: 0.5,
            ..Default::default()
        };
        assert!(e.is_positive());
    }

    #[test]
    fn is_positive_detects_surprise() {
        let e = PerceivedEmotion {
            surprise: 0.5,
            ..Default::default()
        };
        assert!(e.is_positive());
    }

    #[test]
    fn is_positive_false_when_neutral() {
        let e = PerceivedEmotion {
            neutral: 0.9,
            ..Default::default()
        };
        assert!(!e.is_positive());
    }

    #[test]
    fn is_negative_detects_sadness() {
        let e = PerceivedEmotion {
            sadness: 0.5,
            ..Default::default()
        };
        assert!(e.is_negative());
    }

    #[test]
    fn is_negative_detects_anger() {
        let e = PerceivedEmotion {
            anger: 0.6,
            ..Default::default()
        };
        assert!(e.is_negative());
    }

    #[test]
    fn is_negative_detects_fear() {
        let e = PerceivedEmotion {
            fear: 0.5,
            ..Default::default()
        };
        assert!(e.is_negative());
    }

    #[test]
    fn is_negative_false_when_happy() {
        let e = PerceivedEmotion {
            happiness: 0.9,
            ..Default::default()
        };
        assert!(!e.is_negative());
    }

    #[test]
    fn infer_affectionate_when_reaching_and_happy() {
        let e = PerceivedEmotion {
            happiness: 0.5,
            ..Default::default()
        };
        let intent = PerceivedIntent::infer(&e, 1.0, true, true, 0.3);
        assert_eq!(intent, PerceivedIntent::Affectionate);
    }

    #[test]
    fn infer_aggressive_when_angry_and_approaching() {
        let e = PerceivedEmotion {
            anger: 0.7,
            ..Default::default()
        };
        let intent = PerceivedIntent::infer(&e, 2.0, true, false, 0.3);
        assert_eq!(intent, PerceivedIntent::Aggressive);
    }

    #[test]
    fn infer_pleading_when_sad_and_close() {
        let e = PerceivedEmotion {
            sadness: 0.6,
            ..Default::default()
        };
        let intent = PerceivedIntent::infer(&e, 1.5, false, false, 0.2);
        assert_eq!(intent, PerceivedIntent::Pleading);
    }

    #[test]
    fn infer_commanding_when_loud_and_slightly_angry() {
        let e = PerceivedEmotion {
            anger: 0.3,
            ..Default::default()
        };
        let intent = PerceivedIntent::infer(&e, 2.0, false, false, 0.8);
        assert_eq!(intent, PerceivedIntent::Commanding);
    }

    #[test]
    fn infer_retreating_when_far_and_not_approaching() {
        let e = PerceivedEmotion::default();
        let intent = PerceivedIntent::infer(&e, 5.0, false, false, 0.1);
        assert_eq!(intent, PerceivedIntent::Retreating);
    }

    #[test]
    fn infer_friendly_when_happy_and_approaching() {
        let e = PerceivedEmotion {
            happiness: 0.5,
            ..Default::default()
        };
        let intent = PerceivedIntent::infer(&e, 2.0, true, false, 0.3);
        assert_eq!(intent, PerceivedIntent::Friendly);
    }

    #[test]
    fn infer_curious_when_close() {
        let e = PerceivedEmotion::default();
        let intent = PerceivedIntent::infer(&e, 2.0, false, false, 0.1);
        assert_eq!(intent, PerceivedIntent::Curious);
    }

    #[test]
    fn infer_neutral_when_far_and_approaching() {
        let e = PerceivedEmotion::default();
        // Far away but approaching — doesn't match retreating (is_approaching=true),
        // doesn't match friendly (happiness too low). Distance > 3.0 but approaching.
        let intent = PerceivedIntent::infer(&e, 5.0, true, false, 0.1);
        assert_eq!(intent, PerceivedIntent::Neutral);
    }
}
