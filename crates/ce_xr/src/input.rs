use ce_math::{Quat, Vec3};

/// Head tracking data, updated each frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct HeadPose {
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
}

/// Single hand tracking data (controller-level).
#[derive(Debug, Clone, Copy)]
pub struct HandPose {
    pub position: Vec3,
    pub orientation: Quat,
    pub grip_position: Vec3,
    pub grip_orientation: Quat,
    pub trigger_value: f32,   // 0.0 - 1.0
    pub grip_value: f32,      // 0.0 - 1.0
    pub thumbstick: [f32; 2], // x, y
    pub is_tracked: bool,
}

impl Default for HandPose {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            grip_position: Vec3::ZERO,
            grip_orientation: Quat::IDENTITY,
            trigger_value: 0.0,
            grip_value: 0.0,
            thumbstick: [0.0, 0.0],
            is_tracked: false,
        }
    }
}

/// XR input state, inserted as a Resource.
#[derive(Debug, Clone, Default)]
pub struct XrInput {
    pub head: HeadPose,
    pub left_hand: HandPose,
    pub right_hand: HandPose,
}

// ---------------------------------------------------------------------------
// Fine-grained hand joint tracking (26 joints per hand, OpenXR standard)
// ---------------------------------------------------------------------------

/// Hand joint identifiers following the OpenXR `XR_EXT_hand_tracking` layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum HandJoint {
    Palm = 0,
    Wrist = 1,
    ThumbMetacarpal = 2,
    ThumbProximal = 3,
    ThumbDistal = 4,
    ThumbTip = 5,
    IndexMetacarpal = 6,
    IndexProximal = 7,
    IndexIntermediate = 8,
    IndexDistal = 9,
    IndexTip = 10,
    MiddleMetacarpal = 11,
    MiddleProximal = 12,
    MiddleIntermediate = 13,
    MiddleDistal = 14,
    MiddleTip = 15,
    RingMetacarpal = 16,
    RingProximal = 17,
    RingIntermediate = 18,
    RingDistal = 19,
    RingTip = 20,
    LittleMetacarpal = 21,
    LittleProximal = 22,
    LittleIntermediate = 23,
    LittleDistal = 24,
    LittleTip = 25,
}

/// Per-joint state within a hand skeleton.
#[derive(Debug, Clone, Copy)]
pub struct JointState {
    pub position: Vec3,
    pub orientation: Quat,
    /// Joint radius for collision detection (meters).
    pub radius: f32,
    /// Tracking confidence (0.0 - 1.0).
    pub confidence: f32,
}

impl Default for JointState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            radius: 0.0,
            confidence: 0.0,
        }
    }
}

/// Detailed hand skeleton with per-joint pose (26 joints per OpenXR spec).
#[derive(Debug, Clone)]
pub struct HandSkeleton {
    pub joints: Vec<JointState>,
    pub is_tracked: bool,
}

impl Default for HandSkeleton {
    fn default() -> Self {
        Self {
            joints: vec![JointState::default(); 26],
            is_tracked: false,
        }
    }
}

impl HandSkeleton {
    /// Access a specific joint by its identifier.
    pub fn get_joint(&self, joint: HandJoint) -> &JointState {
        &self.joints[joint as usize]
    }

    /// Detect pinch gesture (thumb tip close to index tip).
    pub fn is_pinching(&self, threshold: f32) -> bool {
        let thumb = self.joints[HandJoint::ThumbTip as usize].position;
        let index = self.joints[HandJoint::IndexTip as usize].position;
        thumb.distance(index) < threshold
    }

    /// Detect fist (all finger tips close to palm).
    pub fn is_fist(&self, threshold: f32) -> bool {
        let palm = self.joints[HandJoint::Palm as usize].position;
        [
            HandJoint::IndexTip,
            HandJoint::MiddleTip,
            HandJoint::RingTip,
            HandJoint::LittleTip,
        ]
        .iter()
        .all(|&j| self.joints[j as usize].position.distance(palm) < threshold)
    }

    /// Detect pointing (index extended, others curled).
    pub fn is_pointing(&self, extend_threshold: f32, curl_threshold: f32) -> bool {
        let palm = self.joints[HandJoint::Palm as usize].position;
        let index_extended = self.joints[HandJoint::IndexTip as usize]
            .position
            .distance(palm)
            > extend_threshold;
        let others_curled = [
            HandJoint::MiddleTip,
            HandJoint::RingTip,
            HandJoint::LittleTip,
        ]
        .iter()
        .all(|&j| self.joints[j as usize].position.distance(palm) < curl_threshold);
        index_extended && others_curled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_skeleton_default_has_26_joints() {
        let hs = HandSkeleton::default();
        assert_eq!(hs.joints.len(), 26);
        assert!(!hs.is_tracked);
    }

    #[test]
    fn is_pinching_with_close_points() {
        let mut hs = HandSkeleton::default();
        hs.joints[HandJoint::ThumbTip as usize].position = Vec3::new(0.0, 0.0, 0.0);
        hs.joints[HandJoint::IndexTip as usize].position = Vec3::new(0.005, 0.0, 0.0);
        assert!(hs.is_pinching(0.01));
    }

    #[test]
    fn is_pinching_false_when_far_apart() {
        let mut hs = HandSkeleton::default();
        hs.joints[HandJoint::ThumbTip as usize].position = Vec3::new(0.0, 0.0, 0.0);
        hs.joints[HandJoint::IndexTip as usize].position = Vec3::new(0.1, 0.0, 0.0);
        assert!(!hs.is_pinching(0.01));
    }

    #[test]
    fn is_fist_detected() {
        let mut hs = HandSkeleton::default();
        // Palm at origin, all fingertips close to palm
        hs.joints[HandJoint::Palm as usize].position = Vec3::new(0.0, 0.0, 0.0);
        for &tip in &[
            HandJoint::IndexTip,
            HandJoint::MiddleTip,
            HandJoint::RingTip,
            HandJoint::LittleTip,
        ] {
            hs.joints[tip as usize].position = Vec3::new(0.01, 0.01, 0.0);
        }
        assert!(hs.is_fist(0.05));
    }

    #[test]
    fn is_fist_false_when_fingers_extended() {
        let mut hs = HandSkeleton::default();
        hs.joints[HandJoint::Palm as usize].position = Vec3::new(0.0, 0.0, 0.0);
        hs.joints[HandJoint::IndexTip as usize].position = Vec3::new(0.2, 0.0, 0.0);
        hs.joints[HandJoint::MiddleTip as usize].position = Vec3::new(0.2, 0.0, 0.0);
        hs.joints[HandJoint::RingTip as usize].position = Vec3::new(0.2, 0.0, 0.0);
        hs.joints[HandJoint::LittleTip as usize].position = Vec3::new(0.2, 0.0, 0.0);
        assert!(!hs.is_fist(0.05));
    }

    #[test]
    fn is_pointing_detected() {
        let mut hs = HandSkeleton::default();
        hs.joints[HandJoint::Palm as usize].position = Vec3::new(0.0, 0.0, 0.0);
        // Index extended
        hs.joints[HandJoint::IndexTip as usize].position = Vec3::new(0.15, 0.0, 0.0);
        // Others curled
        hs.joints[HandJoint::MiddleTip as usize].position = Vec3::new(0.02, 0.0, 0.0);
        hs.joints[HandJoint::RingTip as usize].position = Vec3::new(0.02, 0.0, 0.0);
        hs.joints[HandJoint::LittleTip as usize].position = Vec3::new(0.02, 0.0, 0.0);
        assert!(hs.is_pointing(0.1, 0.05));
    }

    #[test]
    fn is_pointing_false_when_all_extended() {
        let mut hs = HandSkeleton::default();
        hs.joints[HandJoint::Palm as usize].position = Vec3::new(0.0, 0.0, 0.0);
        // All fingers extended
        for &tip in &[
            HandJoint::IndexTip,
            HandJoint::MiddleTip,
            HandJoint::RingTip,
            HandJoint::LittleTip,
        ] {
            hs.joints[tip as usize].position = Vec3::new(0.15, 0.0, 0.0);
        }
        assert!(!hs.is_pointing(0.1, 0.05));
    }

    #[test]
    fn get_joint_returns_correct_data() {
        let mut hs = HandSkeleton::default();
        hs.joints[HandJoint::Wrist as usize].position = Vec3::new(1.0, 2.0, 3.0);
        let wrist = hs.get_joint(HandJoint::Wrist);
        assert_eq!(wrist.position, Vec3::new(1.0, 2.0, 3.0));
    }
}
