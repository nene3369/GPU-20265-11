use ce_math::{Quat, Vec3};

/// Full body joint identifiers (36 joints, Meta Body Tracking API compatible).
///
/// Covers the upper body, arms, and legs. Finger joints are tracked
/// separately via the hand skeleton in [`crate::input::HandSkeleton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum BodyJoint {
    Root = 0,
    Hips = 1,
    SpineLower = 2,
    SpineMiddle = 3,
    SpineUpper = 4,
    Chest = 5,
    Neck = 6,
    Head = 7,

    // Left arm
    LeftShoulder = 8,
    LeftScapula = 9,
    LeftArmUpper = 10,
    LeftArmLower = 11,
    LeftHandWristTwist = 12,
    LeftHandWrist = 13,
    LeftHandPalm = 14,

    // Right arm
    RightShoulder = 15,
    RightScapula = 16,
    RightArmUpper = 17,
    RightArmLower = 18,
    RightHandWristTwist = 19,
    RightHandWrist = 20,
    RightHandPalm = 21,

    // Left leg
    LeftUpperLeg = 22,
    LeftLowerLeg = 23,
    LeftFootAnkleTwist = 24,
    LeftFootAnkle = 25,
    LeftFootSubtalar = 26,
    LeftFootTransverse = 27,
    LeftFootBall = 28,

    // Right leg
    RightUpperLeg = 29,
    RightLowerLeg = 30,
    RightFootAnkleTwist = 31,
    RightFootAnkle = 32,
    RightFootSubtalar = 33,
    RightFootTransverse = 34,
    RightFootBall = 35,
}

/// A single joint's pose and tracking state.
#[derive(Debug, Clone, Copy)]
pub struct JointPose {
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub confidence: f32,
    pub is_tracked: bool,
}

impl Default for JointPose {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            confidence: 0.0,
            is_tracked: false,
        }
    }
}

/// Full body tracking state.
pub struct BodyTracking {
    pub joints: Vec<JointPose>,
    pub is_calibrated: bool,
    pub tracking_mode: BodyTrackingMode,
    pub provider: BodyTrackingProvider,
}

impl Default for BodyTracking {
    fn default() -> Self {
        Self {
            joints: vec![JointPose::default(); 36],
            is_calibrated: false,
            tracking_mode: BodyTrackingMode::UpperBody,
            provider: BodyTrackingProvider::None,
        }
    }
}

impl BodyTracking {
    /// Access a specific joint by its identifier.
    pub fn get_joint(&self, joint: BodyJoint) -> &JointPose {
        &self.joints[joint as usize]
    }

    /// Compute user height from tracked skeleton (head to ankle distance along Y).
    /// Returns `None` if the required joints are not tracked.
    pub fn estimated_height(&self) -> Option<f32> {
        let head = &self.joints[BodyJoint::Head as usize];
        let foot = &self.joints[BodyJoint::LeftFootAnkle as usize];
        if head.is_tracked && foot.is_tracked {
            Some((head.position.y - foot.position.y).abs())
        } else {
            None
        }
    }
}

/// Body tracking mode, determined by available hardware.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BodyTrackingMode {
    /// Head + hands (3-point IK).
    #[default]
    UpperBody,
    /// Upper body + elbow estimation.
    UpperBodyPlus,
    /// Full body with hips + feet (e.g., Vive Trackers, Quest body tracking).
    FullBody,
    /// Full body + finger tracking.
    FullBodyFingers,
}

/// Identifies the body tracking backend in use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BodyTrackingProvider {
    #[default]
    None,
    /// `XR_FB_body_tracking` (Quest 3).
    MetaFB,
    /// SteamVR tracker dongles (Vive Trackers).
    ViveTrackers,
    /// Open-source IMU trackers (SlimeVR).
    SlimeVR,
    /// Software IK from head + hands only.
    ThreePointIK,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_36_joints() {
        let bt = BodyTracking::default();
        assert_eq!(bt.joints.len(), 36);
    }

    #[test]
    fn get_joint_returns_correct_joint() {
        let mut bt = BodyTracking::default();
        bt.joints[BodyJoint::Head as usize].position = Vec3::new(0.0, 1.7, 0.0);
        let head = bt.get_joint(BodyJoint::Head);
        assert_eq!(head.position, Vec3::new(0.0, 1.7, 0.0));
    }

    #[test]
    fn estimated_height_returns_none_when_untracked() {
        let bt = BodyTracking::default();
        assert_eq!(bt.estimated_height(), None);
    }

    #[test]
    fn estimated_height_returns_value_when_tracked() {
        let mut bt = BodyTracking::default();
        bt.joints[BodyJoint::Head as usize].is_tracked = true;
        bt.joints[BodyJoint::Head as usize].position = Vec3::new(0.0, 1.7, 0.0);
        bt.joints[BodyJoint::LeftFootAnkle as usize].is_tracked = true;
        bt.joints[BodyJoint::LeftFootAnkle as usize].position = Vec3::new(0.0, 0.0, 0.0);
        let height = bt.estimated_height().unwrap();
        assert!((height - 1.7).abs() < 0.001);
    }

    #[test]
    fn default_tracking_mode_is_upper_body() {
        let bt = BodyTracking::default();
        assert_eq!(bt.tracking_mode, BodyTrackingMode::UpperBody);
        assert_eq!(bt.provider, BodyTrackingProvider::None);
        assert!(!bt.is_calibrated);
    }
}
