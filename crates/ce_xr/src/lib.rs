pub mod body_tracking;
pub mod eye_tracking;
pub mod face_tracking;
pub mod input;
pub mod session;
pub mod swapchain;
pub mod types;
pub mod voice;

pub use body_tracking::{
    BodyJoint, BodyTracking, BodyTrackingMode, BodyTrackingProvider, JointPose,
};
pub use eye_tracking::{EyeGaze, EyeState, EyeTracking, EyeTrackingProvider};
pub use face_tracking::{DetectedEmotion, FaceBlendShapes, FaceTracking, FaceTrackingProvider};
pub use input::{HandJoint, HandSkeleton, JointState};
pub use input::{HandPose, HeadPose, XrInput};
pub use session::XrSession;
pub use swapchain::XrSwapchain;
pub use types::{Eye, ViewPose, XrConfig};
pub use voice::{SpeechEvent, VoiceActivity, VoiceConfig, VoiceInput, VoiceProvider};

use ce_app::Plugin;

/// XR Plugin - enables VR/AR headset support via OpenXR.
/// Only activates if an OpenXR runtime is available.
#[derive(Default)]
pub struct XrPlugin {
    pub config: XrConfig,
}

impl Plugin for XrPlugin {
    fn build(&self, app: &mut ce_app::App) {
        // Try to initialize OpenXR. If no runtime is available, log and skip.
        match XrSession::try_new(&self.config) {
            Ok(session) => {
                log::info!("OpenXR runtime found: {}", session.runtime_name());
                app.insert_resource(session);
                app.insert_resource(XrInput::default());
                app.insert_resource(FaceTracking::default());
                app.insert_resource(EyeTracking::default());
                app.insert_resource(BodyTracking::default());
                app.insert_resource(VoiceInput::default());
                log::info!("XR mode enabled — stereo rendering active");
            }
            Err(e) => {
                log::warn!("OpenXR not available: {}. Running in desktop mode.", e);
            }
        }
    }
}
