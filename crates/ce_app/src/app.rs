use std::any::TypeId;
use std::collections::HashSet;

use ce_ecs::{CoreStage, Schedule, World};

use crate::plugin::Plugin;
use crate::time::{FixedTime, Time};

/// The top-level application builder and runner.
///
/// `App` owns the ECS [`World`] and [`Schedule`], provides a builder API
/// for registering plugins, systems, and resources, and drives the
/// per-frame update loop.
///
/// **Note:** `App` does *not* own an event loop. The winit integration
/// lives in `ce_window`; `App` only provides the builder and single-frame
/// `update` / `run_once` methods.
pub struct App {
    /// The ECS world that stores all entities, components, and resources.
    pub world: World,
    /// The schedule that organises systems into stages.
    pub schedule: Schedule,
    /// Tracks which plugins have been installed (by TypeId) to prevent
    /// duplicates.
    plugins_installed: HashSet<TypeId>,
}

impl App {
    /// Creates a new `App` with an empty world and schedule.
    ///
    /// A [`Time`] and [`FixedTime`] resource are inserted automatically so
    /// that frame-timing data is always available to systems.
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(Time::new());
        world.insert_resource(FixedTime::default());

        Self {
            world,
            schedule: Schedule::new(),
            plugins_installed: HashSet::new(),
        }
    }

    /// Adds a [`Plugin`] to the app.
    ///
    /// The plugin's [`Plugin::build`] method is called immediately. If a
    /// plugin of the same concrete type has already been added, the
    /// duplicate is silently ignored and a log message is emitted.
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        let type_id = TypeId::of::<P>();
        if self.plugins_installed.contains(&type_id) {
            log::warn!(
                "Plugin `{}` already installed — skipping duplicate.",
                plugin.name()
            );
            return self;
        }
        self.plugins_installed.insert(type_id);
        log::info!("Installing plugin: {}", plugin.name());
        plugin.build(self);
        self
    }

    /// Registers a system in the given [`CoreStage`].
    pub fn add_system(
        &mut self,
        stage: CoreStage,
        system: impl FnMut(&mut World) + Send + 'static,
    ) -> &mut Self {
        self.schedule.add_system(stage, system);
        self
    }

    /// Inserts a global resource into the world.
    pub fn insert_resource<T: ce_ecs::Resource>(&mut self, resource: T) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Performs a single frame update:
    ///
    /// 1. Updates the [`Time`] resource.
    /// 2. Runs the schedule (all stages in order).
    pub fn update(&mut self) {
        // Update frame timing.
        if let Some(time) = self.world.get_resource_mut::<Time>() {
            time.update();
        }
        // Execute all stages.
        self.schedule.run(&mut self.world);
    }

    /// Convenience method that runs exactly one frame and then returns.
    ///
    /// Equivalent to calling [`App::update`] once. Useful in tests and
    /// headless / batch scenarios.
    pub fn run_once(&mut self) {
        self.update();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// A tiny counter resource used across several tests.
    #[derive(Debug, PartialEq)]
    struct Counter(u32);

    /// A marker resource that proves a plugin ran.
    #[derive(Debug, PartialEq)]
    struct PluginMarker(String);

    /// Minimal plugin for testing.
    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(PluginMarker("TestPlugin was here".into()));
        }
    }

    /// Another plugin type so we can test multi-plugin scenarios.
    struct AnotherPlugin;

    impl Plugin for AnotherPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(Counter(0));
        }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[test]
    fn app_new_inserts_time_resource() {
        let app = App::new();
        assert!(
            app.world.get_resource::<Time>().is_some(),
            "App::new must insert a Time resource"
        );
    }

    #[test]
    fn app_new_inserts_fixed_time_resource() {
        let app = App::new();
        assert!(
            app.world.get_resource::<FixedTime>().is_some(),
            "App::new must insert a FixedTime resource"
        );
    }

    #[test]
    fn add_system_and_run_once_executes_system() {
        let mut app = App::new();
        app.insert_resource(Counter(0));

        app.add_system(CoreStage::Update, |world: &mut World| {
            if let Some(c) = world.get_resource_mut::<Counter>() {
                c.0 += 1;
            }
        });

        app.run_once();

        let counter = app.world.get_resource::<Counter>().unwrap();
        assert_eq!(counter.0, 1);
    }

    #[test]
    fn plugin_builds_correctly() {
        let mut app = App::new();
        app.add_plugin(TestPlugin);

        let marker = app.world.get_resource::<PluginMarker>().unwrap();
        assert_eq!(marker.0, "TestPlugin was here");
    }

    #[test]
    fn duplicate_plugin_is_ignored() {
        let mut app = App::new();
        app.add_plugin(TestPlugin);
        app.add_plugin(TestPlugin); // should be silently skipped

        // The marker should still be the original value — not re-built.
        let marker = app.world.get_resource::<PluginMarker>().unwrap();
        assert_eq!(marker.0, "TestPlugin was here");
    }

    #[test]
    fn multiple_different_plugins() {
        let mut app = App::new();
        app.add_plugin(TestPlugin);
        app.add_plugin(AnotherPlugin);

        assert!(app.world.get_resource::<PluginMarker>().is_some());
        assert!(app.world.get_resource::<Counter>().is_some());
    }

    #[test]
    fn systems_mutate_world_correctly() {
        let mut app = App::new();
        app.insert_resource(Counter(10));

        // System 1: double the counter.
        app.add_system(CoreStage::Update, |world: &mut World| {
            if let Some(c) = world.get_resource_mut::<Counter>() {
                c.0 *= 2;
            }
        });

        // System 2: add 1.
        app.add_system(CoreStage::PostUpdate, |world: &mut World| {
            if let Some(c) = world.get_resource_mut::<Counter>() {
                c.0 += 1;
            }
        });

        app.run_once();

        // Update runs first (10 * 2 = 20), then PostUpdate (20 + 1 = 21).
        let counter = app.world.get_resource::<Counter>().unwrap();
        assert_eq!(counter.0, 21);
    }

    #[test]
    fn update_advances_time() {
        let mut app = App::new();
        app.update();

        let time = app.world.get_resource::<Time>().unwrap();
        assert_eq!(time.frame_count(), 1);
    }

    #[test]
    fn run_once_is_equivalent_to_single_update() {
        let mut app = App::new();
        app.run_once();

        let time = app.world.get_resource::<Time>().unwrap();
        assert_eq!(time.frame_count(), 1);
    }

    #[test]
    fn multiple_updates_increment_frame_count() {
        let mut app = App::new();
        for _ in 0..5 {
            app.update();
        }
        let time = app.world.get_resource::<Time>().unwrap();
        assert_eq!(time.frame_count(), 5);
    }

    #[test]
    fn system_execution_order_across_stages() {
        let order = Arc::new(Mutex::new(Vec::<&str>::new()));

        let mut app = App::new();

        let o1 = order.clone();
        app.add_system(CoreStage::PostUpdate, move |_: &mut World| {
            o1.lock().unwrap().push("post_update");
        });

        let o2 = order.clone();
        app.add_system(CoreStage::First, move |_: &mut World| {
            o2.lock().unwrap().push("first");
        });

        let o3 = order.clone();
        app.add_system(CoreStage::Update, move |_: &mut World| {
            o3.lock().unwrap().push("update");
        });

        app.run_once();

        let result = order.lock().unwrap();
        assert_eq!(*result, vec!["first", "update", "post_update"]);
    }

    #[test]
    fn plugin_can_add_systems() {
        struct SystemPlugin;

        impl Plugin for SystemPlugin {
            fn build(&self, app: &mut App) {
                app.insert_resource(Counter(0));
                app.add_system(CoreStage::Update, |world: &mut World| {
                    if let Some(c) = world.get_resource_mut::<Counter>() {
                        c.0 += 42;
                    }
                });
            }
        }

        let mut app = App::new();
        app.add_plugin(SystemPlugin);
        app.run_once();

        let counter = app.world.get_resource::<Counter>().unwrap();
        assert_eq!(counter.0, 42);
    }

    #[test]
    fn insert_resource_is_accessible() {
        let mut app = App::new();
        app.insert_resource(String::from("hello"));

        let s = app.world.get_resource::<String>().unwrap();
        assert_eq!(s.as_str(), "hello");
    }
}
