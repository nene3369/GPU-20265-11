//! The Three Poisons (Tri-visa) — lobha (greed), dosa (hatred), moha (delusion).
//!
//! Models the fundamental unwholesome roots in Buddhist psychology that
//! drive NPC behaviour. Intervention is possible but carries blowback
//! due to recursive entanglement.

/// The three root poisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoisonType {
    /// Lobha — greed / attachment.
    Lobha,
    /// Dosa — hatred / aversion.
    Dosa,
    /// Moha — delusion / ignorance.
    Moha,
}

/// Three poisons state for an NPC. Each value is in \[0, 1\].
#[derive(Debug, Clone, Copy)]
pub struct ThreePoisons {
    /// Greed (0..=1).
    pub lobha: f64,
    /// Hatred (0..=1).
    pub dosa: f64,
    /// Delusion (0..=1).
    pub moha: f64,
}

impl Default for ThreePoisons {
    /// Small residual amounts — even virtuous beings carry seeds.
    fn default() -> Self {
        Self {
            lobha: 0.1,
            dosa: 0.1,
            moha: 0.2,
        }
    }
}

impl ThreePoisons {
    /// Creates an NPC with a high dominant poison (0.9) and moderate
    /// others (0.4).
    pub fn villain(dominant: PoisonType) -> Self {
        let (l, d, m) = match dominant {
            PoisonType::Lobha => (0.9, 0.4, 0.4),
            PoisonType::Dosa => (0.4, 0.9, 0.4),
            PoisonType::Moha => (0.4, 0.4, 0.9),
        };
        Self {
            lobha: l,
            dosa: d,
            moha: m,
        }
    }

    /// Returns the dominant poison type (highest value).
    pub fn dominant(&self) -> PoisonType {
        if self.lobha >= self.dosa && self.lobha >= self.moha {
            PoisonType::Lobha
        } else if self.dosa >= self.lobha && self.dosa >= self.moha {
            PoisonType::Dosa
        } else {
            PoisonType::Moha
        }
    }

    /// Total poison level — average of the three.
    pub fn total(&self) -> f64 {
        (self.lobha + self.dosa + self.moha) / 3.0
    }

    /// Apply an intervention to reduce a target poison.
    ///
    /// The target poison is reduced by 70% of `strength`, but the other
    /// two poisons receive 30% blowback (recursive entanglement — you
    /// cannot simply excise suffering without consequence).
    pub fn apply_intervention(&mut self, target: PoisonType, strength: f64) {
        let reduction = strength * 0.7;
        let blowback = strength * 0.3;

        match target {
            PoisonType::Lobha => {
                self.lobha = (self.lobha - reduction).max(0.0);
                self.dosa = (self.dosa + blowback).min(1.0);
                self.moha = (self.moha + blowback).min(1.0);
            }
            PoisonType::Dosa => {
                self.dosa = (self.dosa - reduction).max(0.0);
                self.lobha = (self.lobha + blowback).min(1.0);
                self.moha = (self.moha + blowback).min(1.0);
            }
            PoisonType::Moha => {
                self.moha = (self.moha - reduction).max(0.0);
                self.lobha = (self.lobha + blowback).min(1.0);
                self.dosa = (self.dosa + blowback).min(1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_small_values() {
        let p = ThreePoisons::default();
        assert!(p.lobha < 0.3);
        assert!(p.dosa < 0.3);
        assert!(p.moha < 0.3);
    }

    #[test]
    fn villain_has_correct_dominant() {
        assert_eq!(
            ThreePoisons::villain(PoisonType::Lobha).dominant(),
            PoisonType::Lobha
        );
        assert_eq!(
            ThreePoisons::villain(PoisonType::Dosa).dominant(),
            PoisonType::Dosa
        );
        assert_eq!(
            ThreePoisons::villain(PoisonType::Moha).dominant(),
            PoisonType::Moha
        );
    }

    #[test]
    fn total_is_average() {
        let p = ThreePoisons {
            lobha: 0.3,
            dosa: 0.6,
            moha: 0.9,
        };
        let expected = (0.3 + 0.6 + 0.9) / 3.0;
        assert!((p.total() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn intervention_reduces_target() {
        let mut p = ThreePoisons::villain(PoisonType::Dosa);
        let before = p.dosa;
        p.apply_intervention(PoisonType::Dosa, 0.5);
        assert!(
            p.dosa < before,
            "Target should decrease: {} >= {}",
            p.dosa,
            before
        );
    }

    #[test]
    fn intervention_has_blowback() {
        let mut p = ThreePoisons {
            lobha: 0.3,
            dosa: 0.8,
            moha: 0.3,
        };
        let lobha_before = p.lobha;
        let moha_before = p.moha;

        p.apply_intervention(PoisonType::Dosa, 0.5);

        assert!(
            p.lobha > lobha_before,
            "Blowback should increase lobha: {} <= {}",
            p.lobha,
            lobha_before
        );
        assert!(
            p.moha > moha_before,
            "Blowback should increase moha: {} <= {}",
            p.moha,
            moha_before
        );
    }

    #[test]
    fn values_stay_clamped() {
        let mut p = ThreePoisons {
            lobha: 0.95,
            dosa: 0.95,
            moha: 0.1,
        };
        // Strong intervention on moha — blowback should not exceed 1.0
        p.apply_intervention(PoisonType::Moha, 1.0);
        assert!(p.lobha <= 1.0);
        assert!(p.dosa <= 1.0);
        assert!(p.moha >= 0.0);
    }
}
