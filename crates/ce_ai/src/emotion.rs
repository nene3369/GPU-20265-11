//! Continuous emotional state for NPCs.
//!
//! Emotions are represented as continuous values rather than discrete
//! enums, allowing smooth blending and nuanced expression. The eight
//! primary emotions follow Plutchik's wheel.

/// Continuous emotional state — each dimension ranges 0..=1.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmotionalState {
    pub joy: f32,
    pub sadness: f32,
    pub anger: f32,
    pub fear: f32,
    pub surprise: f32,
    pub disgust: f32,
    pub trust: f32,
    pub anticipation: f32,
}

impl EmotionalState {
    /// Smoothly blend toward a target emotional state.
    ///
    /// `rate` controls how fast the transition happens (0 = no change,
    /// 1 = instant snap).
    pub fn blend_toward(&mut self, target: &Self, rate: f32) {
        let r = rate.clamp(0.0, 1.0);
        self.joy += (target.joy - self.joy) * r;
        self.sadness += (target.sadness - self.sadness) * r;
        self.anger += (target.anger - self.anger) * r;
        self.fear += (target.fear - self.fear) * r;
        self.surprise += (target.surprise - self.surprise) * r;
        self.disgust += (target.disgust - self.disgust) * r;
        self.trust += (target.trust - self.trust) * r;
        self.anticipation += (target.anticipation - self.anticipation) * r;
    }

    /// Returns the name of the strongest emotion.
    pub fn dominant(&self) -> &'static str {
        let pairs: [(&str, f32); 8] = [
            ("joy", self.joy),
            ("sadness", self.sadness),
            ("anger", self.anger),
            ("fear", self.fear),
            ("surprise", self.surprise),
            ("disgust", self.disgust),
            ("trust", self.trust),
            ("anticipation", self.anticipation),
        ];
        pairs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| *name)
            .unwrap_or("joy")
    }

    /// Overall emotional intensity — average of all dimensions.
    pub fn intensity(&self) -> f32 {
        (self.joy
            + self.sadness
            + self.anger
            + self.fear
            + self.surprise
            + self.disgust
            + self.trust
            + self.anticipation)
            / 8.0
    }
}

/// Emotional weight for episodic memory — how strongly an event is
/// remembered and whether it has been resolved through practice.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmotionalWeight {
    /// Emotional charge of the memory (positive or negative).
    pub charge: f64,
    /// Proximity to a psychic wound (0 = distant, 1 = core wound).
    pub wound_proximity: f64,
    /// How consolidated the memory is (0 = fresh, 1 = deep).
    pub consolidation: f64,
    /// Whether this memory has been resolved through nirodha practice.
    pub nirodha_resolved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        let e = EmotionalState::default();
        assert!((e.intensity() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn blend_toward_moves_state() {
        let mut state = EmotionalState::default();
        let target = EmotionalState {
            joy: 1.0,
            ..Default::default()
        };

        state.blend_toward(&target, 0.5);
        assert!((state.joy - 0.5).abs() < f32::EPSILON);

        // Blend again — should approach further
        state.blend_toward(&target, 0.5);
        assert!((state.joy - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn blend_toward_rate_zero_is_noop() {
        let mut state = EmotionalState {
            anger: 0.3,
            ..Default::default()
        };
        let target = EmotionalState {
            anger: 1.0,
            ..Default::default()
        };
        state.blend_toward(&target, 0.0);
        assert!((state.anger - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn blend_toward_rate_one_is_snap() {
        let mut state = EmotionalState::default();
        let target = EmotionalState {
            fear: 0.8,
            ..Default::default()
        };
        state.blend_toward(&target, 1.0);
        assert!((state.fear - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn dominant_returns_strongest() {
        let e = EmotionalState {
            trust: 0.9,
            joy: 0.1,
            ..Default::default()
        };
        assert_eq!(e.dominant(), "trust");
    }

    #[test]
    fn dominant_with_anger() {
        let e = EmotionalState {
            anger: 0.8,
            fear: 0.2,
            ..Default::default()
        };
        assert_eq!(e.dominant(), "anger");
    }

    #[test]
    fn intensity_is_average() {
        let e = EmotionalState {
            joy: 0.4,
            sadness: 0.4,
            anger: 0.4,
            fear: 0.4,
            surprise: 0.4,
            disgust: 0.4,
            trust: 0.4,
            anticipation: 0.4,
        };
        assert!((e.intensity() - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn emotional_weight_default() {
        let w = EmotionalWeight::default();
        assert!((w.charge - 0.0).abs() < f64::EPSILON);
        assert!(!w.nirodha_resolved);
    }
}
