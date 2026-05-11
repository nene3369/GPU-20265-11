use ce_math::Vec3;

/// Eye gaze data from eye tracking hardware.
///
/// Gaze origin and direction are in head-local space; fixation point
/// is in world space (where both eyes converge).
#[derive(Debug, Clone, Copy)]
pub struct EyeGaze {
    /// Gaze origin in head-local space.
    pub origin: Vec3,
    /// Gaze direction (normalized).
    pub direction: Vec3,
    /// Convergence point (where both eyes focus) in world space.
    pub fixation_point: Vec3,
    /// Pupil dilation (0.0 = contracted, 1.0 = dilated). Useful for NPC reactions.
    pub pupil_dilation: f32,
    /// Confidence of the tracking (0.0 - 1.0).
    pub confidence: f32,
    pub is_tracked: bool,
}

impl Default for EyeGaze {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            direction: Vec3::NEG_Z, // Forward
            fixation_point: Vec3::ZERO,
            pupil_dilation: 0.5,
            confidence: 0.0,
            is_tracked: false,
        }
    }
}

/// Per-eye data.
#[derive(Debug, Clone, Copy)]
pub struct EyeState {
    pub gaze: EyeGaze,
    /// 0.0 = closed, 1.0 = fully open.
    pub openness: f32,
    /// 0.0 = relaxed, 1.0 = squinting.
    pub squeeze: f32,
}

impl Default for EyeState {
    fn default() -> Self {
        Self {
            gaze: EyeGaze::default(),
            openness: 1.0,
            squeeze: 0.0,
        }
    }
}

/// Eye tracking state resource.
#[derive(Debug, Clone, Default)]
pub struct EyeTracking {
    pub left: EyeState,
    pub right: EyeState,
    /// Combined gaze (average of both eyes).
    pub combined_gaze: EyeGaze,
    pub is_supported: bool,
    pub provider: EyeTrackingProvider,
}

/// Identifies the eye tracking backend / OpenXR extension in use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EyeTrackingProvider {
    #[default]
    None,
    /// `XR_EXT_eye_gaze_interaction`.
    OpenXRExt,
    /// `XR_FB_eye_tracking_social`.
    MetaFB,
    /// Tobii integration via HTC Vive.
    TobiiHTCVive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eye_gaze_default_looks_forward() {
        let gaze = EyeGaze::default();
        assert_eq!(gaze.direction, Vec3::NEG_Z);
        assert!(!gaze.is_tracked);
    }

    #[test]
    fn eye_state_default_is_open() {
        let state = EyeState::default();
        assert_eq!(state.openness, 1.0);
        assert_eq!(state.squeeze, 0.0);
    }

    #[test]
    fn eye_tracking_default_not_supported() {
        let et = EyeTracking::default();
        assert!(!et.is_supported);
        assert_eq!(et.provider, EyeTrackingProvider::None);
    }
}
