//! NPC memory of the player — episodic memory with exponential decay.
//!
//! Each interaction creates a [`MemoryEntry`] with emotional valence and
//! importance. Memories decay over time (negative memories decay faster,
//! modelling nirodha — the cessation of suffering). The accumulated
//! sentiment and trust determine the [`Relationship`] level.

/// A single memory entry about the player.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// Description of the event.
    pub event: String,
    /// Emotional valence: -1.0 (negative) to +1.0 (positive).
    pub emotional_valence: f32,
    /// How important this memory is (0.0-1.0).
    pub importance: f32,
    /// In-game time when the event occurred.
    pub timestamp: f64,
    /// How fast this memory fades.
    pub decay_rate: f64,
}

impl MemoryEntry {
    /// Create a positive memory.
    pub fn positive(event: &str, importance: f32, timestamp: f64) -> Self {
        Self {
            event: event.to_string(),
            emotional_valence: 0.7,
            importance,
            timestamp,
            decay_rate: 0.001,
        }
    }

    /// Create a negative memory (decays faster — nirodha).
    pub fn negative(event: &str, importance: f32, timestamp: f64) -> Self {
        Self {
            event: event.to_string(),
            emotional_valence: -0.5,
            importance,
            timestamp,
            decay_rate: 0.002, // negative memories decay faster
        }
    }

    /// Effective strength of this memory at current time.
    pub fn strength(&self, current_time: f64) -> f64 {
        let dt = current_time - self.timestamp;
        self.importance as f64 * (-dt * self.decay_rate).exp()
    }
}

/// Relationship level with the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Relationship {
    /// No history.
    Stranger,
    /// A few interactions.
    Acquaintance,
    /// Regular interactions.
    Familiar,
    /// Trust built up.
    Friend,
    /// Deep bond.
    Companion,
    /// Maximum trust (rare).
    Soulmate,
}

impl Relationship {
    /// Determine relationship from trust level.
    pub fn from_trust(trust: f32) -> Self {
        if trust < 0.1 {
            Relationship::Stranger
        } else if trust < 0.3 {
            Relationship::Acquaintance
        } else if trust < 0.5 {
            Relationship::Familiar
        } else if trust < 0.7 {
            Relationship::Friend
        } else if trust < 0.9 {
            Relationship::Companion
        } else {
            Relationship::Soulmate
        }
    }

    /// Trust threshold for this relationship level.
    pub fn min_trust(&self) -> f32 {
        match self {
            Relationship::Stranger => 0.0,
            Relationship::Acquaintance => 0.1,
            Relationship::Familiar => 0.3,
            Relationship::Friend => 0.5,
            Relationship::Companion => 0.7,
            Relationship::Soulmate => 0.9,
        }
    }
}

/// NPC's memory of and relationship with the player.
#[derive(Debug, Clone)]
pub struct NpcMemory {
    pub entries: Vec<MemoryEntry>,
    pub trust: f32,
    pub total_interaction_time: f64,
    pub interaction_count: u32,
    pub first_met_time: Option<f64>,
}

impl Default for NpcMemory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            trust: 0.0,
            total_interaction_time: 0.0,
            interaction_count: 0,
            first_met_time: None,
        }
    }
}

impl NpcMemory {
    /// Current relationship level.
    pub fn relationship(&self) -> Relationship {
        Relationship::from_trust(self.trust)
    }

    /// Add a memory and update trust accordingly.
    pub fn add_memory(&mut self, entry: MemoryEntry) {
        self.trust =
            (self.trust + entry.emotional_valence * entry.importance * 0.1).clamp(0.0, 1.0);
        self.entries.push(entry);
    }

    /// Record that an interaction occurred.
    pub fn record_interaction(&mut self, duration: f64, game_time: f64) {
        if self.first_met_time.is_none() {
            self.first_met_time = Some(game_time);
        }
        self.total_interaction_time += duration;
        self.interaction_count += 1;
    }

    /// Overall sentiment (weighted average of memory valences).
    pub fn sentiment(&self, current_time: f64) -> f64 {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        for entry in &self.entries {
            let w = entry.strength(current_time);
            weighted_sum += entry.emotional_valence as f64 * w;
            total_weight += w;
        }
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Prune memories that have decayed below threshold.
    pub fn prune(&mut self, current_time: f64, threshold: f64) {
        self.entries
            .retain(|e| e.strength(current_time) > threshold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_memory_is_stranger() {
        let m = NpcMemory::default();
        assert_eq!(m.relationship(), Relationship::Stranger);
        assert_eq!(m.trust, 0.0);
        assert_eq!(m.interaction_count, 0);
        assert!(m.first_met_time.is_none());
    }

    #[test]
    fn add_positive_memory_increases_trust() {
        let mut m = NpcMemory::default();
        let entry = MemoryEntry::positive("Player helped me", 0.8, 100.0);
        m.add_memory(entry);
        assert!(m.trust > 0.0, "Trust should increase for positive memory");
    }

    #[test]
    fn add_negative_memory_does_not_go_below_zero() {
        let mut m = NpcMemory::default();
        let entry = MemoryEntry::negative("Player was rude", 0.9, 100.0);
        m.add_memory(entry);
        assert!(m.trust >= 0.0, "Trust should not go below 0");
    }

    #[test]
    fn sentiment_decays_over_time() {
        // With two memories of differing decay rates, the weighted average
        // (sentiment) shifts as the faster-decaying memory loses influence.
        let mut m = NpcMemory::default();
        // Positive memory with slow decay
        m.entries.push(MemoryEntry {
            event: "Good event".to_string(),
            emotional_valence: 0.8,
            importance: 0.8,
            timestamp: 0.0,
            decay_rate: 0.001,
        });
        // Negative memory with fast decay
        m.entries.push(MemoryEntry {
            event: "Bad event".to_string(),
            emotional_valence: -0.6,
            importance: 0.8,
            timestamp: 0.0,
            decay_rate: 0.01,
        });

        // Early: both memories strong, sentiment is a mix
        let sentiment_early = m.sentiment(10.0);
        // Late: negative memory has decayed away, positive dominates more
        let sentiment_late = m.sentiment(5000.0);
        assert!(
            sentiment_late > sentiment_early,
            "Sentiment should become more positive as negative memory decays faster: early={}, late={}",
            sentiment_early,
            sentiment_late,
        );
    }

    #[test]
    fn sentiment_is_zero_with_no_memories() {
        let m = NpcMemory::default();
        assert_eq!(m.sentiment(100.0), 0.0);
    }

    #[test]
    fn prune_removes_old_memories() {
        let mut m = NpcMemory::default();
        m.entries.push(MemoryEntry {
            event: "Ancient event".to_string(),
            emotional_valence: 0.5,
            importance: 0.3,
            timestamp: 0.0,
            decay_rate: 0.01,
        });
        m.entries.push(MemoryEntry {
            event: "Recent event".to_string(),
            emotional_valence: 0.5,
            importance: 0.9,
            timestamp: 9990.0,
            decay_rate: 0.001,
        });

        // At time 10000: ancient event strength = 0.3 * exp(-10000*0.01) ~ 0
        // Recent event strength = 0.9 * exp(-10*0.001) ~ 0.891
        m.prune(10000.0, 0.01);
        assert_eq!(m.entries.len(), 1, "Ancient memory should be pruned");
        assert_eq!(m.entries[0].event, "Recent event");
    }

    #[test]
    fn record_interaction_sets_first_met() {
        let mut m = NpcMemory::default();
        m.record_interaction(5.0, 100.0);
        assert_eq!(m.first_met_time, Some(100.0));
        assert_eq!(m.interaction_count, 1);
        assert_eq!(m.total_interaction_time, 5.0);
    }

    #[test]
    fn record_interaction_does_not_overwrite_first_met() {
        let mut m = NpcMemory::default();
        m.record_interaction(5.0, 100.0);
        m.record_interaction(3.0, 200.0);
        assert_eq!(m.first_met_time, Some(100.0));
        assert_eq!(m.interaction_count, 2);
        assert_eq!(m.total_interaction_time, 8.0);
    }

    #[test]
    fn relationship_from_trust_thresholds() {
        assert_eq!(Relationship::from_trust(0.0), Relationship::Stranger);
        assert_eq!(Relationship::from_trust(0.05), Relationship::Stranger);
        assert_eq!(Relationship::from_trust(0.1), Relationship::Acquaintance);
        assert_eq!(Relationship::from_trust(0.2), Relationship::Acquaintance);
        assert_eq!(Relationship::from_trust(0.3), Relationship::Familiar);
        assert_eq!(Relationship::from_trust(0.4), Relationship::Familiar);
        assert_eq!(Relationship::from_trust(0.5), Relationship::Friend);
        assert_eq!(Relationship::from_trust(0.6), Relationship::Friend);
        assert_eq!(Relationship::from_trust(0.7), Relationship::Companion);
        assert_eq!(Relationship::from_trust(0.8), Relationship::Companion);
        assert_eq!(Relationship::from_trust(0.9), Relationship::Soulmate);
        assert_eq!(Relationship::from_trust(1.0), Relationship::Soulmate);
    }

    #[test]
    fn min_trust_is_consistent_with_from_trust() {
        for &(level, expected_min) in &[
            (Relationship::Stranger, 0.0),
            (Relationship::Acquaintance, 0.1),
            (Relationship::Familiar, 0.3),
            (Relationship::Friend, 0.5),
            (Relationship::Companion, 0.7),
            (Relationship::Soulmate, 0.9),
        ] {
            assert_eq!(level.min_trust(), expected_min);
            assert_eq!(Relationship::from_trust(expected_min), level);
        }
    }

    #[test]
    fn memory_strength_decreases_over_time() {
        let entry = MemoryEntry::positive("event", 1.0, 0.0);
        let s1 = entry.strength(0.0);
        let s2 = entry.strength(100.0);
        let s3 = entry.strength(1000.0);
        assert!(s1 > s2, "Strength should decrease: {} <= {}", s1, s2);
        assert!(s2 > s3, "Strength should decrease: {} <= {}", s2, s3);
    }

    #[test]
    fn negative_memories_decay_faster_than_positive() {
        let pos = MemoryEntry::positive("good", 0.8, 0.0);
        let neg = MemoryEntry::negative("bad", 0.8, 0.0);
        // At a far future time, negative should have decayed more
        let t = 500.0;
        let pos_strength = pos.strength(t);
        let neg_strength = neg.strength(t);
        assert!(
            neg_strength < pos_strength,
            "Negative memories should decay faster: neg={}, pos={}",
            neg_strength,
            pos_strength,
        );
    }
}
