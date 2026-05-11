//! Social distance system — proxemics-aware NPC behaviour.
//!
//! Based on Edward T. Hall's proxemics model, classifies the distance
//! between the player and an NPC into social zones (intimate, personal,
//! social, public). NPCs react differently depending on whether the
//! player's proximity exceeds their comfort zone for the current
//! relationship level.

/// Social distance categories (Edward T. Hall's proxemics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum SocialDistance {
    /// < 0.5m — touching, whispering.
    Intimate,
    /// 0.5-1.2m — close friends.
    Personal,
    /// 1.2-3.6m — acquaintances.
    Social,
    /// > 3.6m — strangers.
    #[default]
    Public,
}

impl SocialDistance {
    /// Classify a distance in meters into a social zone.
    pub fn from_distance(meters: f32) -> Self {
        if meters < 0.5 {
            SocialDistance::Intimate
        } else if meters < 1.2 {
            SocialDistance::Personal
        } else if meters < 3.6 {
            SocialDistance::Social
        } else {
            SocialDistance::Public
        }
    }

    /// Is this distance closer than the NPC's comfort zone?
    pub fn is_uncomfortable(&self, relationship: &super::memory::Relationship) -> bool {
        matches!(
            (self, relationship),
            (
                SocialDistance::Intimate,
                super::memory::Relationship::Stranger
            ) | (
                SocialDistance::Intimate,
                super::memory::Relationship::Acquaintance
            ) | (
                SocialDistance::Personal,
                super::memory::Relationship::Stranger
            )
        )
    }
}

/// Per-NPC proximity tracking.
#[derive(Debug, Clone, Default)]
pub struct ProximityState {
    pub distance: f32,
    pub social_zone: SocialDistance,
    pub is_approaching: bool,
    pub approach_speed: f32,
    pub time_in_zone: f32,
    pub previous_distance: f32,
}

impl ProximityState {
    /// Update proximity from a new distance measurement.
    pub fn update(&mut self, new_distance: f32, dt: f32) {
        self.is_approaching = new_distance < self.previous_distance;
        self.approach_speed = (self.previous_distance - new_distance) / dt.max(0.001);
        let new_zone = SocialDistance::from_distance(new_distance);
        if new_zone == self.social_zone {
            self.time_in_zone += dt;
        } else {
            self.time_in_zone = 0.0;
        }
        self.previous_distance = self.distance;
        self.distance = new_distance;
        self.social_zone = new_zone;
    }
}

/// System that updates proximity for all NPCs.
pub fn update_proximity(world: &mut ce_ecs::World) {
    // Placeholder: In full implementation, this queries all NPC entities
    // with ProximityState and updates based on player position.
    let _ = world;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Relationship;

    #[test]
    fn from_distance_intimate() {
        assert_eq!(SocialDistance::from_distance(0.3), SocialDistance::Intimate);
    }

    #[test]
    fn from_distance_personal() {
        assert_eq!(SocialDistance::from_distance(0.8), SocialDistance::Personal);
    }

    #[test]
    fn from_distance_social() {
        assert_eq!(SocialDistance::from_distance(2.0), SocialDistance::Social);
    }

    #[test]
    fn from_distance_public() {
        assert_eq!(SocialDistance::from_distance(5.0), SocialDistance::Public);
    }

    #[test]
    fn from_distance_boundary_intimate_personal() {
        // Exactly 0.5 should be Personal (not Intimate)
        assert_eq!(SocialDistance::from_distance(0.5), SocialDistance::Personal);
    }

    #[test]
    fn from_distance_boundary_personal_social() {
        assert_eq!(SocialDistance::from_distance(1.2), SocialDistance::Social);
    }

    #[test]
    fn from_distance_boundary_social_public() {
        assert_eq!(SocialDistance::from_distance(3.6), SocialDistance::Public);
    }

    #[test]
    fn uncomfortable_stranger_intimate() {
        assert!(SocialDistance::Intimate.is_uncomfortable(&Relationship::Stranger));
    }

    #[test]
    fn uncomfortable_acquaintance_intimate() {
        assert!(SocialDistance::Intimate.is_uncomfortable(&Relationship::Acquaintance));
    }

    #[test]
    fn uncomfortable_stranger_personal() {
        assert!(SocialDistance::Personal.is_uncomfortable(&Relationship::Stranger));
    }

    #[test]
    fn comfortable_friend_intimate() {
        assert!(!SocialDistance::Intimate.is_uncomfortable(&Relationship::Friend));
    }

    #[test]
    fn comfortable_stranger_social() {
        assert!(!SocialDistance::Social.is_uncomfortable(&Relationship::Stranger));
    }

    #[test]
    fn comfortable_stranger_public() {
        assert!(!SocialDistance::Public.is_uncomfortable(&Relationship::Stranger));
    }

    #[test]
    fn proximity_update_detects_approach() {
        let mut ps = ProximityState {
            distance: 5.0,
            previous_distance: 5.0,
            social_zone: SocialDistance::Public,
            ..Default::default()
        };

        // Move closer
        ps.update(3.0, 0.016);
        assert!(
            ps.is_approaching,
            "Should detect approach when distance decreases"
        );
        assert!(ps.approach_speed > 0.0, "Approach speed should be positive");
        assert_eq!(ps.social_zone, SocialDistance::Social);
    }

    #[test]
    fn proximity_update_detects_retreat() {
        let mut ps = ProximityState {
            distance: 2.0,
            previous_distance: 2.0,
            social_zone: SocialDistance::Social,
            ..Default::default()
        };

        // Move farther
        ps.update(4.0, 0.016);
        assert!(
            !ps.is_approaching,
            "Should detect retreat when distance increases"
        );
        assert!(
            ps.approach_speed < 0.0,
            "Approach speed should be negative when retreating"
        );
    }

    #[test]
    fn proximity_update_accumulates_time_in_zone() {
        let mut ps = ProximityState {
            distance: 2.0,
            previous_distance: 2.0,
            social_zone: SocialDistance::Social,
            time_in_zone: 0.0,
            ..Default::default()
        };

        ps.update(2.5, 0.016);
        assert!(ps.time_in_zone > 0.0, "Should accumulate time in same zone");
        let t1 = ps.time_in_zone;

        ps.update(2.8, 0.016);
        assert!(ps.time_in_zone > t1, "Should continue accumulating");
    }

    #[test]
    fn proximity_update_resets_time_on_zone_change() {
        let mut ps = ProximityState {
            distance: 2.0,
            previous_distance: 2.0,
            social_zone: SocialDistance::Social,
            time_in_zone: 5.0,
            ..Default::default()
        };

        // Move to a different zone
        ps.update(0.3, 0.016);
        assert_eq!(ps.time_in_zone, 0.0, "Time should reset on zone change");
        assert_eq!(ps.social_zone, SocialDistance::Intimate);
    }

    #[test]
    fn default_proximity_state() {
        let ps = ProximityState::default();
        assert_eq!(ps.distance, 0.0);
        assert_eq!(ps.social_zone, SocialDistance::Public);
        assert!(!ps.is_approaching);
    }
}
