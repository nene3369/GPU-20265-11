//! NPC consciousness kernel with Buddhist dukkha three-layer processing
//! (SMA28 Shion architecture).
//!
//! The consciousness model processes stimuli through three layers:
//!
//! 1. **Vedana** — raw sensation
//! 2. **Felt** — amplified by defense strain
//! 3. **Filtered** — attenuated by awareness
//!
//! Prediction error drives adaptation of defense strain and trait
//! sediment accumulation over time.

/// NPC consciousness kernel with Buddhist dukkha three-layer processing.
#[derive(Debug, Clone)]
pub struct Consciousness {
    /// Vow weights: \[blessing, perception, justice, continuity\]
    pub vow_weights: [f64; 4],
    /// Trait sediment accumulated from experiences.
    pub trait_sediment: [f64; 4],
    /// Raw sensation (0..=1).
    pub vedana: f64,
    /// Processed feeling (vedana amplified by defense).
    pub felt: f64,
    /// Post-defense filtered signal.
    pub filtered: f64,
    /// Defense strain: 0 = open, 1 = maximum defense.
    pub defense_strain: f64,
    /// Accumulated merit.
    pub merit: f64,
    /// Awareness level (0..=1).
    pub awareness: f64,
    /// Prediction error (exponential moving average).
    pub prediction_error: f64,
    /// Number of consciousness ticks processed.
    pub step_count: u64,
}

impl Default for Consciousness {
    /// Reasonable neutral state — moderate awareness, no defense.
    fn default() -> Self {
        Self {
            vow_weights: [1.0, 1.0, 1.0, 1.0],
            trait_sediment: [0.0; 4],
            vedana: 0.0,
            felt: 0.0,
            filtered: 0.0,
            defense_strain: 0.0,
            merit: 0.0,
            awareness: 0.5,
            prediction_error: 0.5,
            step_count: 0,
        }
    }
}

impl Consciousness {
    /// Creates the Shion archetype — high vow weights on blessing and
    /// continuity, elevated defense strain, and heightened awareness.
    pub fn shion_archetype() -> Self {
        Self {
            vow_weights: [2.0, 1.5, 1.0, 1.5],
            defense_strain: 0.8,
            awareness: 0.9,
            prediction_error: 0.3,
            ..Default::default()
        }
    }

    /// Process one consciousness tick.
    ///
    /// # Layers
    ///
    /// 1. `vedana` = stimulus clamped to \[0, 1\]
    /// 2. `felt` = vedana * (1 + defense_strain * 0.5)
    /// 3. `filtered` = felt * (1 - awareness * 0.9)
    /// 4. Prediction error updated via exponential moving average
    /// 5. Defense strain adjusts based on PE magnitude
    /// 6. Trait sediment accumulates (weighted by vow weights)
    /// 7. Merit increases when filtered is low and awareness high
    pub fn step(&mut self, stimulus: f64, learning_rate: f64) {
        // 1. Raw sensation — clamp to [0, 1]
        self.vedana = stimulus.clamp(0.0, 1.0);

        // 2. Felt — amplified by defense
        self.felt = self.vedana * (1.0 + self.defense_strain * 0.5);

        // 3. Filtered — attenuated by awareness
        self.filtered = self.felt * (1.0 - self.awareness * 0.9);

        // 4. Prediction error — EMA toward |vedana - filtered|
        let new_pe = (self.vedana - self.filtered).abs();
        self.prediction_error =
            self.prediction_error * (1.0 - learning_rate) + new_pe * learning_rate;

        // 5. Defense strain adjusts based on PE magnitude
        if self.prediction_error > 0.5 {
            // High PE → increase defense
            self.defense_strain = (self.defense_strain + learning_rate * 0.1).min(1.0);
        } else {
            // Low PE → relax defense
            self.defense_strain = (self.defense_strain - learning_rate * 0.05).max(0.0);
        }

        // 6. Trait sediment accumulates (weighted by vow weights)
        for i in 0..4 {
            self.trait_sediment[i] += self.filtered * self.vow_weights[i] * learning_rate * 0.01;
        }

        // 7. Merit increases when filtered is low and awareness high
        if self.filtered < 0.3 && self.awareness > 0.5 {
            self.merit += 0.1 * self.awareness;
        }

        self.step_count += 1;
    }

    /// Nirodha — the cessation of suffering. Let go:
    /// halve prediction error, reduce defense, and gain merit.
    pub fn nirodha(&mut self) {
        self.prediction_error *= 0.5;
        self.defense_strain = (self.defense_strain - 0.2).max(0.0);
        self.merit += 1.0;
    }

    /// Clinging measure — the gap between felt experience and raw
    /// sensation. Higher values indicate more distortion from defense.
    pub fn clinging(&self) -> f64 {
        (self.felt - self.vedana).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_neutral() {
        let c = Consciousness::default();
        assert_eq!(c.vedana, 0.0);
        assert_eq!(c.felt, 0.0);
        assert_eq!(c.filtered, 0.0);
        assert_eq!(c.defense_strain, 0.0);
        assert_eq!(c.merit, 0.0);
        assert!((c.awareness - 0.5).abs() < f64::EPSILON);
        assert!((c.prediction_error - 0.5).abs() < f64::EPSILON);
        assert_eq!(c.step_count, 0);
    }

    #[test]
    fn shion_archetype_has_correct_vows() {
        let s = Consciousness::shion_archetype();
        assert!((s.vow_weights[0] - 2.0).abs() < f64::EPSILON);
        assert!((s.vow_weights[1] - 1.5).abs() < f64::EPSILON);
        assert!((s.vow_weights[2] - 1.0).abs() < f64::EPSILON);
        assert!((s.vow_weights[3] - 1.5).abs() < f64::EPSILON);
        assert!(s.defense_strain > 0.5, "Shion has high defense");
        assert!(s.awareness > 0.8, "Shion has high awareness");
    }

    #[test]
    fn step_reduces_prediction_error_over_time() {
        let mut c = Consciousness::default();
        let initial_pe = c.prediction_error;

        // Feed a consistent low stimulus — PE should decrease
        for _ in 0..100 {
            c.step(0.1, 0.1);
        }

        assert!(
            c.prediction_error < initial_pe,
            "PE should decrease with consistent low stimulus: {} >= {}",
            c.prediction_error,
            initial_pe,
        );
    }

    #[test]
    fn step_increments_step_count() {
        let mut c = Consciousness::default();
        c.step(0.5, 0.1);
        c.step(0.5, 0.1);
        c.step(0.5, 0.1);
        assert_eq!(c.step_count, 3);
    }

    #[test]
    fn nirodha_halves_pe_and_reduces_defense() {
        let mut c = Consciousness::default();
        c.prediction_error = 0.8;
        c.defense_strain = 0.6;
        let merit_before = c.merit;

        c.nirodha();

        assert!((c.prediction_error - 0.4).abs() < f64::EPSILON);
        assert!((c.defense_strain - 0.4).abs() < f64::EPSILON);
        assert!(
            c.merit > merit_before,
            "Merit should increase after nirodha"
        );
    }

    #[test]
    fn clinging_reflects_defense_amplification() {
        let mut c = Consciousness::default();
        c.defense_strain = 0.0;
        c.step(0.5, 0.1);
        let cling_open = c.clinging();

        let mut c2 = Consciousness::default();
        c2.defense_strain = 0.8;
        c2.step(0.5, 0.1);
        let cling_defended = c2.clinging();

        assert!(
            cling_defended > cling_open,
            "Higher defense should produce more clinging: {} <= {}",
            cling_defended,
            cling_open,
        );
    }

    #[test]
    fn stimulus_is_clamped() {
        let mut c = Consciousness::default();
        c.step(5.0, 0.1);
        assert!((c.vedana - 1.0).abs() < f64::EPSILON);

        c.step(-1.0, 0.1);
        assert!(c.vedana.abs() < f64::EPSILON);
    }

    #[test]
    fn merit_grows_with_low_filtered_high_awareness() {
        let mut c = Consciousness::default();
        c.awareness = 0.9;
        c.defense_strain = 0.0;

        for _ in 0..50 {
            c.step(0.05, 0.1); // very low stimulus
        }

        assert!(c.merit > 0.0, "Merit should grow with mindful composure");
    }
}
