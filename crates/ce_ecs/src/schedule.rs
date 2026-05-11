use crate::world::World;

/// Execution stages that define the order in which systems run each frame.
///
/// Systems registered to earlier stages run before those in later stages.
/// Within a single stage, systems run in insertion order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreStage {
    /// Runs before everything else. Setup, resource loading.
    First,
    /// Runs before the main update. Input processing, pre-physics.
    PreUpdate,
    /// Fixed-timestep update for physics and deterministic simulation.
    FixedUpdate,
    /// The main game logic update stage.
    Update,
    /// Runs after the main update. Cleanup, post-physics.
    PostUpdate,
    /// Last logic stage before rendering.
    Last,
    /// Render stage. Presentation and GPU submission.
    Render,
}

impl CoreStage {
    /// Returns all stages in execution order.
    fn all_in_order() -> &'static [CoreStage] {
        &[
            CoreStage::First,
            CoreStage::PreUpdate,
            CoreStage::FixedUpdate,
            CoreStage::Update,
            CoreStage::PostUpdate,
            CoreStage::Last,
            CoreStage::Render,
        ]
    }
}

/// Boxed system function: a `FnMut(&mut World)` closure that can modify the
/// world each time it runs.
type SystemFn = Box<dyn FnMut(&mut World) + Send>;

/// A schedule that organizes systems into stages and runs them in order.
///
/// Systems within the same stage execute sequentially in insertion order.
/// Stages execute in the fixed order defined by [`CoreStage`].
pub struct Schedule {
    /// Pairs of (stage, systems-in-that-stage) in execution order.
    stages: Vec<(CoreStage, Vec<SystemFn>)>,
}

impl Schedule {
    /// Creates a new schedule with all [`CoreStage`] variants in order,
    /// each initially containing no systems.
    pub fn new() -> Self {
        let stages = CoreStage::all_in_order()
            .iter()
            .map(|&stage| (stage, Vec::new()))
            .collect();
        Self { stages }
    }

    /// Adds a system to the given stage.
    ///
    /// The system will run in the order it was added relative to other
    /// systems in the same stage.
    pub fn add_system(
        &mut self,
        stage: CoreStage,
        system: impl FnMut(&mut World) + Send + 'static,
    ) {
        for (s, systems) in &mut self.stages {
            if *s == stage {
                systems.push(Box::new(system));
                return;
            }
        }
    }

    /// Runs all systems in stage order.
    ///
    /// Within each stage, systems execute sequentially in the order they
    /// were added.
    pub fn run(&mut self, world: &mut World) {
        for (_stage, systems) in &mut self.stages {
            for system in systems.iter_mut() {
                system(world);
            }
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn systems_run_in_stage_order() {
        let order = Arc::new(Mutex::new(Vec::new()));

        let mut schedule = Schedule::new();

        let o1 = order.clone();
        schedule.add_system(CoreStage::Update, move |_world: &mut World| {
            o1.lock().unwrap().push("update");
        });

        let o2 = order.clone();
        schedule.add_system(CoreStage::First, move |_world: &mut World| {
            o2.lock().unwrap().push("first");
        });

        let o3 = order.clone();
        schedule.add_system(CoreStage::Last, move |_world: &mut World| {
            o3.lock().unwrap().push("last");
        });

        let o4 = order.clone();
        schedule.add_system(CoreStage::PreUpdate, move |_world: &mut World| {
            o4.lock().unwrap().push("pre_update");
        });

        let mut world = World::new();
        schedule.run(&mut world);

        let result = order.lock().unwrap();
        assert_eq!(*result, vec!["first", "pre_update", "update", "last"]);
    }

    #[test]
    fn multiple_systems_same_stage_run_in_insertion_order() {
        let order = Arc::new(Mutex::new(Vec::new()));

        let mut schedule = Schedule::new();

        let o1 = order.clone();
        schedule.add_system(CoreStage::Update, move |_world: &mut World| {
            o1.lock().unwrap().push(1);
        });

        let o2 = order.clone();
        schedule.add_system(CoreStage::Update, move |_world: &mut World| {
            o2.lock().unwrap().push(2);
        });

        let o3 = order.clone();
        schedule.add_system(CoreStage::Update, move |_world: &mut World| {
            o3.lock().unwrap().push(3);
        });

        let mut world = World::new();
        schedule.run(&mut world);

        let result = order.lock().unwrap();
        assert_eq!(*result, vec![1, 2, 3]);
    }

    #[test]
    fn system_can_mutate_world() {
        let mut schedule = Schedule::new();

        schedule.add_system(CoreStage::Update, |world: &mut World| {
            world.insert_resource(42u32);
        });

        schedule.add_system(CoreStage::PostUpdate, |world: &mut World| {
            if let Some(val) = world.get_resource_mut::<u32>() {
                *val += 1;
            }
        });

        let mut world = World::new();
        schedule.run(&mut world);

        assert_eq!(world.get_resource::<u32>(), Some(&43));
    }

    #[test]
    fn empty_schedule_runs_without_error() {
        let mut schedule = Schedule::new();
        let mut world = World::new();
        schedule.run(&mut world); // Should not panic.
    }

    #[test]
    fn system_spawns_entities() {
        let mut schedule = Schedule::new();

        schedule.add_system(CoreStage::Update, |world: &mut World| {
            for _ in 0..10 {
                world.spawn();
            }
        });

        let mut world = World::new();
        schedule.run(&mut world);
        assert_eq!(world.entity_count(), 10);
    }
}
