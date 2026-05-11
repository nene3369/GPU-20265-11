use crate::app::App;

/// A modular piece of functionality that can be added to an [`App`].
///
/// Plugins encapsulate setup logic — registering systems, inserting
/// resources, and configuring stages — so that features are
/// self-contained and composable.
///
/// # Example
///
/// ```ignore
/// struct PhysicsPlugin;
///
/// impl Plugin for PhysicsPlugin {
///     fn build(&self, app: &mut App) {
///         app.insert_resource(Gravity(9.81))
///            .add_system(CoreStage::FixedUpdate, physics_step);
///     }
/// }
/// ```
pub trait Plugin: 'static + Send + Sync {
    /// Called once when the plugin is added to the app. Register systems,
    /// insert resources, and perform any other one-time setup here.
    fn build(&self, app: &mut App);

    /// Human-readable name used for logging and duplicate-detection
    /// diagnostics. Defaults to the Rust type name.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
