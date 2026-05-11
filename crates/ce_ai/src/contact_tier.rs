//! NPC knowledge provenance — tracks what an NPC knows, how they
//! learned it, and how reliable that knowledge is.
//!
//! Inspired by the Buddhist emphasis on direct experience vs hearsay:
//! first-hand knowledge from direct contact is most reliable, while
//! hearsay degrades confidence and freshness.

/// The tier of contact through which knowledge was acquired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactTier {
    /// Direct experience — the NPC witnessed or experienced this.
    FirstHand,
    /// Derived from reasoning or inference.
    Derived,
    /// Heard from another entity — lowest reliability.
    Hearsay,
}

/// A single piece of knowledge held by an NPC.
#[derive(Debug, Clone)]
pub struct Knowledge {
    /// What the NPC knows (free-form content).
    pub content: String,
    /// How the knowledge was acquired.
    pub tier: ContactTier,
    /// Subjective confidence in this knowledge (0..=1).
    pub confidence: f64,
    /// How fresh the knowledge is (1 = just learned, decays over time).
    pub freshness: f64,
    /// Who or what provided this knowledge (if applicable).
    pub source: Option<String>,
}

impl Knowledge {
    /// Create first-hand knowledge — maximum confidence and freshness.
    pub fn first_hand(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            tier: ContactTier::FirstHand,
            confidence: 1.0,
            freshness: 1.0,
            source: None,
        }
    }

    /// Create hearsay knowledge — reduced confidence, attributed source.
    pub fn hearsay(content: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            tier: ContactTier::Hearsay,
            confidence: 0.5,
            freshness: 1.0,
            source: Some(source.into()),
        }
    }

    /// Advance time — freshness decays exponentially with a half-life
    /// around 300 seconds (exp(-dt/300)).
    pub fn tick(&mut self, dt: f64) {
        self.freshness *= (-dt / 300.0_f64).exp();
    }

    /// Whether the NPC would confidently assert this knowledge.
    ///
    /// Requires both sufficient confidence (> 0.7) and freshness (> 0.3).
    pub fn would_assert(&self) -> bool {
        self.confidence > 0.7 && self.freshness > 0.3
    }
}

/// A collection of knowledge entries held by an NPC.
#[derive(Debug, Clone, Default)]
pub struct KnowledgeBase {
    /// All knowledge entries.
    pub entries: Vec<Knowledge>,
}

impl KnowledgeBase {
    /// Add a knowledge entry.
    pub fn add(&mut self, knowledge: Knowledge) {
        self.entries.push(knowledge);
    }

    /// Count of first-hand knowledge entries.
    pub fn first_hand_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|k| k.tier == ContactTier::FirstHand)
            .count()
    }

    /// Returns all knowledge entries that the NPC would confidently assert.
    pub fn assertable_knowledge(&self) -> Vec<&Knowledge> {
        self.entries.iter().filter(|k| k.would_assert()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_hand_has_full_confidence() {
        let k = Knowledge::first_hand("The sky is blue");
        assert!((k.confidence - 1.0).abs() < f64::EPSILON);
        assert_eq!(k.tier, ContactTier::FirstHand);
        assert!(k.source.is_none());
    }

    #[test]
    fn hearsay_has_reduced_confidence() {
        let k = Knowledge::hearsay("Dragons exist", "Old Man Jenkins");
        assert!((k.confidence - 0.5).abs() < f64::EPSILON);
        assert_eq!(k.tier, ContactTier::Hearsay);
        assert_eq!(k.source.as_deref(), Some("Old Man Jenkins"));
    }

    #[test]
    fn tick_decays_freshness() {
        let mut k = Knowledge::first_hand("Recent event");
        assert!((k.freshness - 1.0).abs() < f64::EPSILON);

        k.tick(300.0); // One half-life-ish period
                       // exp(-1) ≈ 0.368
        assert!(k.freshness < 0.5, "Freshness should decay: {}", k.freshness);
        assert!(
            k.freshness > 0.2,
            "Should not decay too far: {}",
            k.freshness
        );
    }

    #[test]
    fn tick_preserves_freshness_at_zero_dt() {
        let mut k = Knowledge::first_hand("Timeless truth");
        k.tick(0.0);
        assert!((k.freshness - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn would_assert_first_hand_fresh() {
        let k = Knowledge::first_hand("I saw the sun");
        assert!(k.would_assert());
    }

    #[test]
    fn would_not_assert_hearsay() {
        let k = Knowledge::hearsay("Someone said something", "Unknown");
        // confidence is 0.5, below 0.7 threshold
        assert!(!k.would_assert());
    }

    #[test]
    fn would_not_assert_stale() {
        let mut k = Knowledge::first_hand("Old news");
        // Decay until freshness drops below 0.3
        k.tick(600.0); // exp(-2) ≈ 0.135
        assert!(!k.would_assert(), "Stale knowledge should not be asserted");
    }

    #[test]
    fn knowledge_base_add_and_count() {
        let mut kb = KnowledgeBase::default();
        kb.add(Knowledge::first_hand("Fact A"));
        kb.add(Knowledge::first_hand("Fact B"));
        kb.add(Knowledge::hearsay("Rumour C", "Traveller"));

        assert_eq!(kb.entries.len(), 3);
        assert_eq!(kb.first_hand_count(), 2);
    }

    #[test]
    fn assertable_knowledge_filters_correctly() {
        let mut kb = KnowledgeBase::default();
        kb.add(Knowledge::first_hand("Assertable fact"));
        kb.add(Knowledge::hearsay("Non-assertable rumour", "Source"));

        let mut stale = Knowledge::first_hand("Stale fact");
        stale.tick(1000.0); // very stale
        kb.add(stale);

        let assertable = kb.assertable_knowledge();
        assert_eq!(assertable.len(), 1);
        assert_eq!(assertable[0].content, "Assertable fact");
    }
}
