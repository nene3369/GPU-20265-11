pub mod camera;
pub mod hierarchy;
pub mod name;
pub mod transform;

pub use camera::{Camera, CameraBundle, Projection};
pub use hierarchy::{Children, Parent};
pub use name::Name;
pub use transform::{GlobalTransform, Transform};

use ce_app::{App, Plugin};
use ce_ecs::CoreStage;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(CoreStage::PostUpdate, transform::propagate_transforms);
        log::info!("ScenePlugin loaded — transform propagation active");
    }
}
