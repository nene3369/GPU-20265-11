use crate::types::{Eye, Fov, ViewPose, XrConfig};
use ce_math::{Mat4, Quat, Vec3};

/// Manages the OpenXR session lifecycle.
/// Inserted as a Resource in the ECS World when XR is available.
pub struct XrSession {
    runtime_name: String,
    is_running: bool,
    #[allow(dead_code)] // Will be used in M2 frame-timing integration
    predicted_display_time: u64,
    view_poses: [ViewPose; 2],
    recommended_resolution: (u32, u32),
}

impl XrSession {
    /// Try to create an OpenXR session. Returns Err if no runtime is available.
    pub fn try_new(config: &XrConfig) -> Result<Self, String> {
        // Attempt to load OpenXR runtime
        // SAFETY: Entry::load dynamically loads the OpenXR loader library.
        // This is safe as long as the loader library (if present) is a valid
        // OpenXR implementation, which is guaranteed by the OpenXR spec.
        let entry = unsafe { openxr::Entry::load() }
            .map_err(|e| format!("OpenXR runtime not found: {}", e))?;

        // Query runtime properties
        let instance = entry
            .create_instance(
                &openxr::ApplicationInfo {
                    application_name: &config.app_name,
                    application_version: 1,
                    engine_name: "ChemEngine",
                    engine_version: 1,
                    api_version: openxr::CURRENT_API_VERSION,
                },
                &openxr::ExtensionSet::default(),
                &[],
            )
            .map_err(|e| format!("Failed to create OpenXR instance: {}", e))?;

        let props = instance
            .properties()
            .map_err(|e| format!("Failed to get runtime properties: {}", e))?;

        let runtime_name = props.runtime_name.to_string();

        // For now, return a session object with default poses.
        // Full Vulkan graphics binding integration happens in M2.
        let default_fov = Fov::symmetric(1.57, 1.57); // ~90 degrees
        let default_pose = |eye: Eye| ViewPose {
            eye,
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            fov: default_fov,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: default_fov.to_projection_matrix(0.01, 1000.0),
        };

        Ok(Self {
            runtime_name,
            is_running: false,
            predicted_display_time: 0,
            view_poses: [default_pose(Eye::Left), default_pose(Eye::Right)],
            recommended_resolution: (2064, 2096), // Quest 3 default
        })
    }

    pub fn runtime_name(&self) -> &str {
        &self.runtime_name
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn view_poses(&self) -> &[ViewPose; 2] {
        &self.view_poses
    }

    pub fn recommended_resolution(&self) -> (u32, u32) {
        self.recommended_resolution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_without_runtime_returns_error() {
        // On machines without an OpenXR runtime, this should return Err, not panic.
        let config = XrConfig::default();
        let result = XrSession::try_new(&config);
        // We don't assert Ok or Err — either is valid depending on the machine.
        // The important thing is it doesn't panic.
        match result {
            Ok(session) => {
                assert!(!session.runtime_name().is_empty());
            }
            Err(e) => {
                assert!(e.contains("OpenXR") || e.contains("runtime"));
            }
        }
    }
}
