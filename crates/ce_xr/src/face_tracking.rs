/// 63 facial blend shapes (ARKit-compatible).
/// Each value is 0.0 (neutral) to 1.0 (fully activated).
///
/// Models the superset of blend shapes supported by:
/// - Meta Quest Pro/3 via `XR_FB_face_tracking2`
/// - HTC Vive via `XR_HTC_facial_tracking`
/// - Apple ARKit (reference standard)
#[derive(Debug, Clone, Copy)]
pub struct FaceBlendShapes {
    // Eye
    pub eye_blink_left: f32,
    pub eye_blink_right: f32,
    pub eye_look_down_left: f32,
    pub eye_look_down_right: f32,
    pub eye_look_in_left: f32,
    pub eye_look_in_right: f32,
    pub eye_look_out_left: f32,
    pub eye_look_out_right: f32,
    pub eye_look_up_left: f32,
    pub eye_look_up_right: f32,
    pub eye_squint_left: f32,
    pub eye_squint_right: f32,
    pub eye_wide_left: f32,
    pub eye_wide_right: f32,

    // Jaw
    pub jaw_forward: f32,
    pub jaw_left: f32,
    pub jaw_open: f32,
    pub jaw_right: f32,

    // Mouth
    pub mouth_close: f32,
    pub mouth_dimple_left: f32,
    pub mouth_dimple_right: f32,
    pub mouth_frown_left: f32,
    pub mouth_frown_right: f32,
    pub mouth_funnel: f32,
    pub mouth_left: f32,
    pub mouth_lower_down_left: f32,
    pub mouth_lower_down_right: f32,
    pub mouth_press_left: f32,
    pub mouth_press_right: f32,
    pub mouth_pucker: f32,
    pub mouth_right: f32,
    pub mouth_roll_lower: f32,
    pub mouth_roll_upper: f32,
    pub mouth_shrug_lower: f32,
    pub mouth_shrug_upper: f32,
    pub mouth_smile_left: f32,
    pub mouth_smile_right: f32,
    pub mouth_stretch_left: f32,
    pub mouth_stretch_right: f32,
    pub mouth_upper_up_left: f32,
    pub mouth_upper_up_right: f32,

    // Brow
    pub brow_down_left: f32,
    pub brow_down_right: f32,
    pub brow_inner_up: f32,
    pub brow_outer_up_left: f32,
    pub brow_outer_up_right: f32,

    // Cheek
    pub cheek_puff: f32,
    pub cheek_squint_left: f32,
    pub cheek_squint_right: f32,

    // Nose
    pub nose_sneer_left: f32,
    pub nose_sneer_right: f32,

    // Tongue
    pub tongue_out: f32,
}

impl Default for FaceBlendShapes {
    fn default() -> Self {
        // All neutral (0.0).
        // SAFETY: All fields are f32, for which zeroed memory is valid (0.0f32).
        unsafe { std::mem::zeroed() }
    }
}

impl FaceBlendShapes {
    /// Number of blend shape fields in this struct.
    const FIELD_COUNT: usize = std::mem::size_of::<Self>() / std::mem::size_of::<f32>();

    /// Get blend shape value by index (0-based).
    /// Returns `None` if `index` is out of range.
    pub fn get_by_index(&self, index: usize) -> Option<f32> {
        if index < Self::FIELD_COUNT {
            let ptr = self as *const Self as *const f32;
            // SAFETY: We verified index is within bounds, and the struct
            // is a plain sequence of f32 fields with no padding (repr not
            // specified, but f32-only structs have no padding on all
            // supported platforms).
            Some(unsafe { *ptr.add(index) })
        } else {
            None
        }
    }

    /// Detect basic emotions from blend shapes using simple heuristics.
    ///
    /// For production use, replace with an ML model; this provides a
    /// fast approximation suitable for avatar mirroring.
    pub fn detect_emotion(&self) -> DetectedEmotion {
        let smile = (self.mouth_smile_left + self.mouth_smile_right) * 0.5;
        let frown = (self.mouth_frown_left + self.mouth_frown_right) * 0.5;
        let surprise = (self.eye_wide_left + self.eye_wide_right) * 0.5 + self.jaw_open * 0.5;
        let anger = (self.brow_down_left + self.brow_down_right) * 0.5;
        let sadness = (self.brow_inner_up + frown) * 0.5;

        DetectedEmotion {
            happiness: smile.clamp(0.0, 1.0),
            sadness: sadness.clamp(0.0, 1.0),
            surprise: surprise.clamp(0.0, 1.0),
            anger: anger.clamp(0.0, 1.0),
            neutral: (1.0 - smile - frown - surprise * 0.5 - anger).clamp(0.0, 1.0),
            confidence: 0.8, // Placeholder; real confidence comes from ML model
        }
    }
}

/// Detected emotional state from facial expressions.
#[derive(Debug, Clone, Copy, Default)]
pub struct DetectedEmotion {
    pub happiness: f32,
    pub sadness: f32,
    pub surprise: f32,
    pub anger: f32,
    pub neutral: f32,
    pub confidence: f32,
}

/// Face tracking state, inserted as a Resource.
#[derive(Debug, Clone, Default)]
pub struct FaceTracking {
    pub blend_shapes: FaceBlendShapes,
    pub emotion: DetectedEmotion,
    pub is_tracked: bool,
    /// Which face tracking extension is in use.
    pub provider: FaceTrackingProvider,
}

/// Identifies the face tracking backend / OpenXR extension in use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FaceTrackingProvider {
    #[default]
    None,
    /// `XR_FB_face_tracking2` (Quest Pro/3).
    MetaFB,
    /// `XR_HTC_facial_tracking` (Vive).
    HtcSR,
    /// Future standard extension.
    Generic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_zeroed() {
        let shapes = FaceBlendShapes::default();
        let ptr = &shapes as *const FaceBlendShapes as *const f32;
        for i in 0..FaceBlendShapes::FIELD_COUNT {
            let val = unsafe { *ptr.add(i) };
            assert_eq!(val, 0.0, "field index {} should be 0.0", i);
        }
    }

    #[test]
    fn detect_emotion_smile_returns_high_happiness() {
        let mut shapes = FaceBlendShapes::default();
        shapes.mouth_smile_left = 0.9;
        shapes.mouth_smile_right = 0.9;
        let emotion = shapes.detect_emotion();
        assert!(
            emotion.happiness > 0.8,
            "happiness should be high for a smile"
        );
    }

    #[test]
    fn detect_emotion_neutral_returns_high_neutral() {
        let shapes = FaceBlendShapes::default();
        let emotion = shapes.detect_emotion();
        assert!(
            emotion.neutral > 0.8,
            "neutral should be high when all shapes are zero, got {}",
            emotion.neutral
        );
    }

    #[test]
    fn get_by_index_valid() {
        let mut shapes = FaceBlendShapes::default();
        shapes.eye_blink_left = 0.42;
        assert_eq!(shapes.get_by_index(0), Some(0.42));
    }

    #[test]
    fn get_by_index_out_of_range_returns_none() {
        let shapes = FaceBlendShapes::default();
        assert_eq!(shapes.get_by_index(9999), None);
    }

    #[test]
    fn face_tracking_default_is_untracked() {
        let ft = FaceTracking::default();
        assert!(!ft.is_tracked);
        assert_eq!(ft.provider, FaceTrackingProvider::None);
    }
}
