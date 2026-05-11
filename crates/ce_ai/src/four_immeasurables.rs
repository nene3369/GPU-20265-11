//! The Four Immeasurables (Brahma-viharas) as a reward function.
//!
//! Models the Buddhist ideal of universal benevolence as a scalar
//! reward signal for NPC decision-making:
//!
//! - **Maitri** (loving-kindness) — rewards increasing others' happiness
//! - **Karuna** (compassion) — rewards decreasing others' suffering
//! - **Mudita** (sympathetic joy) — amplifies reward from shared happiness
//! - **Upekkha** (equanimity) — stabilises reward regardless of outcome

/// The Four Immeasurables — Buddhist reward function for NPC behaviour.
#[derive(Debug, Clone, Copy)]
pub struct FourImmeasurables {
    /// Maitri — loving-kindness (0..=1).
    pub maitri: f64,
    /// Karuna — compassion (0..=1).
    pub karuna: f64,
    /// Mudita — sympathetic joy (0..=1).
    pub mudita: f64,
    /// Upekkha — equanimity (0..=1).
    pub upekkha: f64,
}

impl Default for FourImmeasurables {
    /// Balanced baseline — moderate cultivation in all four.
    fn default() -> Self {
        Self {
            maitri: 0.5,
            karuna: 0.5,
            mudita: 0.5,
            upekkha: 0.5,
        }
    }
}

impl FourImmeasurables {
    /// The bodhisattva ideal — all immeasurables at maximum.
    pub fn bodhisattva() -> Self {
        Self {
            maitri: 1.0,
            karuna: 1.0,
            mudita: 1.0,
            upekkha: 1.0,
        }
    }

    /// Compute a reward signal from changes in happiness and suffering.
    ///
    /// - `happiness_delta` — positive means others became happier
    /// - `suffering_delta` — positive means others' suffering increased
    ///
    /// The reward formula:
    /// ```text
    /// reward = maitri * happiness_delta
    ///        - karuna * suffering_delta
    ///        + mudita * max(happiness_delta, 0)
    ///        + upekkha * 0.1   (equanimity bonus — steady baseline)
    /// ```
    pub fn compute_reward(&self, happiness_delta: f64, suffering_delta: f64) -> f64 {
        let kindness_term = self.maitri * happiness_delta;
        let compassion_term = self.karuna * suffering_delta;
        let joy_term = self.mudita * happiness_delta.max(0.0);
        let equanimity_term = self.upekkha * 0.1;

        kindness_term - compassion_term + joy_term + equanimity_term
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let fi = FourImmeasurables::default();
        assert!((fi.maitri - 0.5).abs() < f64::EPSILON);
        assert!((fi.karuna - 0.5).abs() < f64::EPSILON);
        assert!((fi.mudita - 0.5).abs() < f64::EPSILON);
        assert!((fi.upekkha - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn bodhisattva_all_one() {
        let b = FourImmeasurables::bodhisattva();
        assert!((b.maitri - 1.0).abs() < f64::EPSILON);
        assert!((b.karuna - 1.0).abs() < f64::EPSILON);
        assert!((b.mudita - 1.0).abs() < f64::EPSILON);
        assert!((b.upekkha - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reward_positive_when_helping() {
        let fi = FourImmeasurables::bodhisattva();
        // Others became happier, suffering decreased
        let reward = fi.compute_reward(1.0, -0.5);
        assert!(
            reward > 0.0,
            "Reward should be positive when helping: {}",
            reward
        );
    }

    #[test]
    fn reward_negative_when_harming() {
        let fi = FourImmeasurables::bodhisattva();
        // Others became less happy, suffering increased
        let reward = fi.compute_reward(-1.0, 2.0);
        assert!(
            reward < 0.0,
            "Reward should be negative when harming: {}",
            reward
        );
    }

    #[test]
    fn equanimity_provides_baseline() {
        let fi = FourImmeasurables::bodhisattva();
        // Neutral scenario — no change in happiness or suffering
        let reward = fi.compute_reward(0.0, 0.0);
        // Only equanimity term remains: 1.0 * 0.1 = 0.1
        assert!(
            reward > 0.0,
            "Equanimity should provide a positive baseline: {}",
            reward
        );
        assert!((reward - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn higher_compassion_penalises_suffering_more() {
        let low_karuna = FourImmeasurables {
            karuna: 0.1,
            ..Default::default()
        };
        let high_karuna = FourImmeasurables {
            karuna: 0.9,
            ..Default::default()
        };

        let r_low = low_karuna.compute_reward(0.0, 1.0);
        let r_high = high_karuna.compute_reward(0.0, 1.0);

        assert!(
            r_high < r_low,
            "Higher karuna should penalise suffering more: {} >= {}",
            r_high,
            r_low
        );
    }
}
