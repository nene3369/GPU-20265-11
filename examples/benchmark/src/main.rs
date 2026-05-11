/// ChemEngine Performance Benchmark
/// Measures throughput and latency of all major subsystems.
use std::time::Instant;

use ce_ai::{Consciousness, FourImmeasurables};
use ce_chemistry::{ElementId, PeriodicTable, ReactionRegistry};
use ce_ecs::World;
use ce_interaction::memory::{MemoryEntry, NpcMemory};
use ce_math::Vec3;
use ce_physics::{BodyType, PhysicsConfig, RigidBody, Velocity};
use ce_render::gpu_cull::{GpuAabb, GpuCullPipeline};
use ce_render::render_graph::RenderGraph;
use ce_scene::Transform;
use ce_worldgen::{BiomeMap, Dungeon, TerrainChunk, TerrainConfig};

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   ChemEngine Performance Benchmark                   ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    bench_ecs_spawn();
    bench_ecs_query();
    bench_ecs_insert_component();
    bench_ecs_for_each();
    bench_ecs_entities_with();
    bench_physics_integration();
    bench_physics_scaling();
    bench_physics_entities_with2_counter_example();
    bench_gpu_physics_cpu_fallback();
    bench_worldgen_terrain();
    bench_worldgen_dungeon();
    bench_ai_consciousness();
    bench_ai_scaling();
    bench_chemistry_lookup();
    bench_chemistry_reactions();
    bench_interaction_memory();
    bench_gpu_cull_cpu();
    bench_gpu_draw_list();
    bench_draw_list_preallocated();
    bench_render_graph();
    bench_transform_matrix();

    println!();
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   Benchmark complete                                 ║");
    println!("╚══════════════════════════════════════════════════════╝");
}

// ─────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────

fn bench<F: FnMut()>(name: &str, iterations: u64, mut f: F) {
    // Warmup
    for _ in 0..3 {
        f();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();

    // Use f64 to avoid integer-division-to-zero for sub-nanosecond ops
    // (e.g. array-indexed lookups can be <1ns on modern CPUs).
    let elapsed_ns = elapsed.as_nanos() as f64;
    let iter_f = iterations as f64;
    let per_iter_ns = if iter_f > 0.0 { elapsed_ns / iter_f } else { 0.0 };
    let ops_per_sec = if per_iter_ns > 0.0 {
        (1_000_000_000.0 / per_iter_ns) as u128
    } else {
        u128::MAX
    };

    println!(
        "  {:<40} {:>12}/iter  {:>12} ops/s",
        name,
        format_ns(per_iter_ns),
        format_num(ops_per_sec)
    );
}

fn bench_once<F: FnMut()>(name: &str, mut f: F) {
    let start = Instant::now();
    f();
    let elapsed = start.elapsed();
    println!("  {:<40} {:>10?}", name, elapsed);
}

fn format_ns(ns: f64) -> String {
    if ns < 1.0 {
        format!("{:.3}ns", ns)
    } else if ns < 1_000.0 {
        format!("{:.0}ns", ns)
    } else if ns < 1_000_000.0 {
        format!("{:.3}µs", ns / 1_000.0)
    } else if ns < 1_000_000_000.0 {
        format!("{:.3}ms", ns / 1_000_000.0)
    } else {
        format!("{:.3}s", ns / 1_000_000_000.0)
    }
}

fn format_num(n: u128) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1e9)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1e6)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1e3)
    } else {
        format!("{}", n)
    }
}

// ─────────────────────────────────────────────────
// ECS Benchmarks
// ─────────────────────────────────────────────────

fn bench_ecs_spawn() {
    println!("━━━ ECS: Entity Spawn ━━━");

    for &count in &[1_000, 10_000, 100_000] {
        let label = format!("spawn {} entities", count);
        bench_once(&label, || {
            let mut world = World::new();
            for _ in 0..count {
                world.spawn();
            }
        });
    }
    println!();
}

fn bench_ecs_query() {
    println!("━━━ ECS: Query Iteration ━━━");

    for &count in &[1_000, 10_000, 100_000] {
        let mut world = World::new();
        for i in 0..count {
            let e = world.spawn();
            world.insert_component(
                e,
                Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            );
        }

        let label = format!("query<Transform> {} entities", count);
        bench(&label, 100, || {
            let results = world.query::<Transform>();
            std::hint::black_box(results.len());
        });
    }
    println!();
}

fn bench_ecs_insert_component() {
    println!("━━━ ECS: Insert Component (Archetype Migration) ━━━");

    let label = "spawn + insert 2 components × 10K";
    bench_once(label, || {
        let mut world = World::new();
        for _ in 0..10_000 {
            let e = world.spawn();
            world.insert_component(e, Transform::default());
            world.insert_component(e, Velocity::default());
        }
    });
    println!();
}

fn bench_ecs_for_each() {
    println!("━━━ ECS: for_each (zero-alloc) ━━━");

    for &count in &[1_000, 10_000, 100_000] {
        let mut world = World::new();
        for i in 0..count {
            let e = world.spawn();
            world.insert_component(
                e,
                Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            );
        }

        let label = format!("for_each<Transform> {} entities", count);
        bench(&label, 100, || {
            let mut count = 0u64;
            world.for_each::<Transform, _>(|_e, _t| {
                count += 1;
            });
            std::hint::black_box(count);
        });
    }
    println!();
}

fn bench_ecs_entities_with() {
    println!("━━━ ECS: entities_with (ID-only) ━━━");

    for &count in &[1_000, 10_000, 100_000] {
        let mut world = World::new();
        for i in 0..count {
            let e = world.spawn();
            world.insert_component(
                e,
                Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            );
        }

        let label = format!("entities_with<Transform> {} entities", count);
        bench(&label, 100, || {
            let ids = world.entities_with::<Transform>();
            std::hint::black_box(ids.len());
        });
    }
    println!();
}

// ─────────────────────────────────────────────────
// Physics Benchmarks
// ─────────────────────────────────────────────────

fn bench_physics_integration() {
    println!("━━━ Physics: Semi-Implicit Euler Integration ━━━");

    let mut world = World::new();
    world.insert_resource(PhysicsConfig {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        fixed_timestep: 1.0 / 60.0,
    });

    for i in 0..1_000 {
        let e = world.spawn();
        world.insert_component(
            e,
            Transform::from_translation(Vec3::new(i as f32, 100.0, 0.0)),
        );
        world.insert_component(e, RigidBody::default());
        world.insert_component(e, Velocity::default());
    }

    bench("1K bodies × 1 physics step", 1000, || {
        // Manual integration (same as physics_step)
        let dt = 1.0 / 60.0_f32;
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let bodies: Vec<_> = {
            let r = world.query2::<RigidBody, Velocity>();
            r.iter().map(|&(e, rb, vel)| (e, *rb, *vel)).collect()
        };
        for (entity, rb, mut vel) in bodies {
            if rb.body_type != BodyType::Dynamic {
                continue;
            }
            vel.linear += gravity * dt;
            vel.linear *= 1.0 - rb.linear_damping;
            if let Some(tf) = world.get_component::<Transform>(entity).copied() {
                let mut new_tf = tf;
                new_tf.translation += vel.linear * dt;
                world.insert_component(entity, new_tf);
            }
            world.insert_component(entity, vel);
        }
    });
    println!();
}

fn bench_physics_scaling() {
    println!("━━━ Physics: Scaling (entity count) ━━━");

    for &count in &[100, 1_000, 10_000, 50_000] {
        let mut world = World::new();
        let dt = 1.0 / 60.0_f32;
        let gravity = Vec3::new(0.0, -9.81, 0.0);

        for i in 0..count {
            let e = world.spawn();
            world.insert_component(
                e,
                Transform::from_translation(Vec3::new(i as f32, 100.0, 0.0)),
            );
            world.insert_component(e, RigidBody::default());
            world.insert_component(e, Velocity::default());
        }

        let label = format!("{} bodies × 1 step", count);
        bench(&label, 10, || {
            let bodies: Vec<_> = {
                let r = world.query2::<RigidBody, Velocity>();
                r.iter().map(|&(e, rb, vel)| (e, *rb, *vel)).collect()
            };
            for (entity, rb, mut vel) in bodies {
                if rb.body_type != BodyType::Dynamic {
                    continue;
                }
                vel.linear += gravity * dt;
                vel.linear *= 1.0 - rb.linear_damping;
                if let Some(tf) = world.get_component::<Transform>(entity).copied() {
                    let mut new_tf = tf;
                    new_tf.translation += vel.linear * dt;
                    world.insert_component(entity, new_tf);
                }
                world.insert_component(entity, vel);
            }
        });
    }
    println!();
}

/// Counter-example: `entities_with2 + get_component` is SLOWER than `query2`.
///
/// Despite being historically labeled "optimized", this pattern does 5 HashMap
/// operations per entity (3 gets, 2 inserts) because `entities_with2` only
/// returns IDs — every component must then be re-looked-up per entity. In
/// contrast, `query2` walks the archetype ONCE and returns `(Entity, &RB, &Vel)`
/// tuples directly, giving effectively 0 lookups per entity (+ 2 inserts for
/// the write-back).
///
/// Measured regression at 50K bodies: `query2` ≈ 9.8 ms, this path ≈ 12.7 ms
/// (≈ +25 %). This benchmark is kept deliberately to document the anti-pattern
/// — do NOT copy this loop shape into production physics systems.
fn bench_physics_entities_with2_counter_example() {
    println!("━━━ Physics: entities_with2 counter-example (SLOWER than query2) ━━━");

    for &count in &[100, 1_000, 10_000, 50_000] {
        let mut world = World::new();
        let dt = 1.0 / 60.0_f32;
        let gravity = Vec3::new(0.0, -9.81, 0.0);

        for i in 0..count {
            let e = world.spawn();
            world.insert_component(
                e,
                Transform::from_translation(Vec3::new(i as f32, 100.0, 0.0)),
            );
            world.insert_component(e, RigidBody::default());
            world.insert_component(e, Velocity::default());
        }

        let label = format!("{} bodies (anti-pattern)", count);
        bench(&label, 10, || {
            let ids = world.entities_with2::<RigidBody, Velocity>();
            for entity in ids {
                let rb = match world.get_component::<RigidBody>(entity).copied() {
                    Some(r) => r,
                    None => continue,
                };
                if rb.body_type != BodyType::Dynamic {
                    continue;
                }

                let mut vel = match world.get_component::<Velocity>(entity).copied() {
                    Some(v) => v,
                    None => continue,
                };
                vel.linear += gravity * dt;
                vel.linear *= 1.0 - rb.linear_damping;

                if let Some(tf) = world.get_component::<Transform>(entity).copied() {
                    let mut new_tf = tf;
                    new_tf.translation += vel.linear * dt;
                    world.insert_component(entity, new_tf);
                }
                world.insert_component(entity, vel);
            }
        });
    }
    println!();
}

// ─────────────────────────────────────────────────
// GPU Physics (CPU Fallback) Benchmarks
// ─────────────────────────────────────────────────

fn bench_gpu_physics_cpu_fallback() {
    println!("━━━ Physics: GPU Format (CPU fallback) ━━━");
    use ce_physics::gpu_physics::{GpuBody, GpuPhysics, GpuPhysicsParams};

    let params = GpuPhysicsParams {
        gravity: [0.0, -9.81, 0.0],
        dt: 1.0 / 60.0,
    };

    for &count in &[1_000, 10_000, 50_000, 100_000, 500_000, 1_000_000] {
        let mut bodies: Vec<GpuBody> = (0..count)
            .map(|i| GpuBody {
                pos: [i as f32 * 0.1, 100.0, 0.0],
                mass: 1.0,
                vel: [0.0; 3],
                damping: 0.01,
                body_type: 1,
                _pad: [0; 3],
            })
            .collect();

        let label = format!("{} bodies (serial)", count);
        bench(&label, 100, || {
            GpuPhysics::integrate_cpu(&mut bodies, &params);
        });
    }
    println!();

    println!("━━━ Physics: GPU Format (rayon parallel) ━━━");

    for &count in &[1_000, 10_000, 50_000, 100_000, 500_000, 1_000_000] {
        let mut bodies: Vec<GpuBody> = (0..count)
            .map(|i| GpuBody {
                pos: [i as f32 * 0.1, 100.0, 0.0],
                mass: 1.0,
                vel: [0.0; 3],
                damping: 0.01,
                body_type: 1,
                _pad: [0; 3],
            })
            .collect();

        let label = format!("{} bodies (parallel)", count);
        bench(&label, 100, || {
            GpuPhysics::integrate_cpu_parallel(&mut bodies, &params);
        });
    }
    println!();
}

// ─────────────────────────────────────────────────
// WorldGen Benchmarks
// ─────────────────────────────────────────────────

fn bench_worldgen_terrain() {
    println!("━━━ WorldGen: Terrain Generation ━━━");

    for &size in &[32, 64, 128, 256] {
        let config = TerrainConfig {
            seed: 42,
            chunk_size: size,
            ..Default::default()
        };
        let label = format!("{}x{} terrain chunk", size, size);
        bench(&label, 100, || {
            let chunk = TerrainChunk::generate(&config, 0, 0);
            std::hint::black_box(&chunk);
        });
    }

    let config = TerrainConfig {
        seed: 42,
        chunk_size: 64,
        ..Default::default()
    };
    bench("64x64 terrain + biome map", 100, || {
        let chunk = TerrainChunk::generate(&config, 0, 0);
        let biome = BiomeMap::generate(&chunk, 42, 0.35);
        std::hint::black_box(&biome);
    });
    println!();
}

fn bench_worldgen_dungeon() {
    println!("━━━ WorldGen: Dungeon Generation ━━━");

    for &(w, h) in &[(40, 30), (80, 50), (160, 100), (320, 200)] {
        let label = format!("{}x{} dungeon", w, h);
        bench(&label, 100, || {
            let d = Dungeon::generate(w, h, 42, 5);
            std::hint::black_box(&d);
        });
    }
    println!();
}

// ─────────────────────────────────────────────────
// AI Benchmarks
// ─────────────────────────────────────────────────

fn bench_ai_consciousness() {
    println!("━━━ AI: Consciousness Step ━━━");

    let mut c = Consciousness::shion_archetype();
    bench("consciousness.step()", 100_000, || {
        let stim = std::hint::black_box(0.5);
        let lr = std::hint::black_box(0.01);
        c.step(stim, lr);
        std::hint::black_box(&c);
    });

    bench("consciousness.nirodha()", 100_000, || {
        c.nirodha();
        std::hint::black_box(&c);
    });

    let imm = FourImmeasurables::bodhisattva();
    bench("four_immeasurables.compute_reward()", 100_000, || {
        let h = std::hint::black_box(0.5);
        let s = std::hint::black_box(-0.3);
        std::hint::black_box(std::hint::black_box(&imm).compute_reward(h, s));
    });
    println!();
}

fn bench_ai_scaling() {
    println!("━━━ AI: NPC Scaling ━━━");

    for &count in &[100, 1_000, 10_000] {
        let mut npcs: Vec<Consciousness> = (0..count)
            .map(|_| Consciousness::shion_archetype())
            .collect();

        let label = format!("{} NPCs × 1 consciousness step", count);
        bench(&label, 100, || {
            for c in &mut npcs {
                c.step(0.5, 0.01);
            }
        });
    }
    println!();
}

// ─────────────────────────────────────────────────
// Chemistry Benchmarks
// ─────────────────────────────────────────────────

fn bench_chemistry_lookup() {
    println!("━━━ Chemistry: Element Lookup ━━━");

    let table = PeriodicTable::new();

    bench("get(ElementId) lookup", 1_000_000, || {
        let id = std::hint::black_box(ElementId(26));
        std::hint::black_box(std::hint::black_box(&table).get(id));
    });

    bench("by_symbol(\"Fe\") lookup", 100_000, || {
        let s = std::hint::black_box("Fe");
        std::hint::black_box(std::hint::black_box(&table).by_symbol(s));
    });
    println!();
}

fn bench_chemistry_reactions() {
    println!("━━━ Chemistry: Reaction Matching ━━━");

    let registry = ReactionRegistry::default();

    bench("find_matching([\"H2\", \"O2\"])", 100_000, || {
        std::hint::black_box(registry.find_matching(&["H2", "O2"]));
    });
    println!();
}

// ─────────────────────────────────────────────────
// Interaction Benchmarks
// ─────────────────────────────────────────────────

fn bench_interaction_memory() {
    println!("━━━ Interaction: NPC Memory ━━━");

    bench("add 1000 memories + sentiment", 1000, || {
        let mut mem = NpcMemory::default();
        for i in 0..1000 {
            mem.add_memory(MemoryEntry::positive("event", 0.5, i as f64));
        }
        std::hint::black_box(mem.sentiment(1000.0));
    });

    bench("prune 1000 memories", 1000, || {
        let mut mem = NpcMemory::default();
        for i in 0..1000 {
            mem.add_memory(MemoryEntry::positive("event", 0.5, i as f64));
        }
        mem.prune(10000.0, 0.01);
        std::hint::black_box(mem.entries.len());
    });
    println!();
}

// ─────────────────────────────────────────────────
// GPU Culling (CPU fallback) Benchmarks
// ─────────────────────────────────────────────────

fn bench_gpu_cull_cpu() {
    println!("━━━ Render: CPU Frustum Culling ━━━");

    let vp: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, -1.001, -1.0],
        [0.0, 0.0, -0.2, 0.0],
    ];
    let frustum = GpuCullPipeline::extract_frustum_planes(&vp);

    for &count in &[1_000, 10_000, 100_000] {
        let aabbs: Vec<GpuAabb> = (0..count)
            .map(|i| {
                let x = (i as f32 * 0.1) % 100.0 - 50.0;
                let z = (i as f32 * 0.07) % 100.0 - 50.0;
                GpuAabb {
                    min: [x - 0.5, -0.5, z - 0.5],
                    _pad0: 0.0,
                    max: [x + 0.5, 0.5, z + 0.5],
                    _pad1: 0.0,
                }
            })
            .collect();

        let label = format!("cull {} AABBs (CPU)", count);
        bench(&label, 100, || {
            let results = GpuCullPipeline::cull_cpu(&frustum, &aabbs);
            std::hint::black_box(&results);
        });
    }
    println!();
}

fn bench_gpu_draw_list() {
    println!("━━━ Render: GPU Draw List ━━━");

    use ce_render::gpu_driven::{GpuDrawList, ObjectData};

    for &count in &[1_000, 10_000, 100_000] {
        let label = format!("build {} indirect commands", count);
        bench(&label, 10, || {
            let mut list = GpuDrawList::new();
            for i in 0..count {
                list.add_object(ObjectData {
                    model: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                    aabb_min: [-0.5, -0.5, -0.5],
                    _pad0: 0.0,
                    aabb_max: [0.5, 0.5, 0.5],
                    _pad1: 0.0,
                    mesh_id: 0,
                    vertex_count: 36,
                    first_vertex: i * 36,
                    _pad2: 0,
                });
            }
            std::hint::black_box(list.object_count());
        });
    }
    println!();
}

fn bench_draw_list_preallocated() {
    println!("━━━ Render: GPU Draw List (pre-allocated) ━━━");
    use ce_render::gpu_driven::{GpuDrawList, ObjectData};

    for &count in &[1_000, 10_000, 100_000] {
        let label = format!("build {} commands (pre-alloc)", count);
        bench(&label, 10, || {
            let mut list = GpuDrawList::with_capacity(count as usize);
            for i in 0..count {
                list.add_object(ObjectData {
                    model: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                    aabb_min: [-0.5, -0.5, -0.5],
                    _pad0: 0.0,
                    aabb_max: [0.5, 0.5, 0.5],
                    _pad1: 0.0,
                    mesh_id: 0,
                    vertex_count: 36,
                    first_vertex: i * 36,
                    _pad2: 0,
                });
            }
            std::hint::black_box(list.object_count());
        });
    }
    println!();
}

// ─────────────────────────────────────────────────
// RenderGraph Benchmark
// ─────────────────────────────────────────────────

fn bench_render_graph() {
    println!("━━━ Render: RenderGraph Compile ━━━");

    for &pass_count in &[5, 20, 50] {
        let label = format!("compile {} passes (DAG)", pass_count);
        bench(&label, 1000, || {
            let mut graph = RenderGraph::new();
            let mut passes = Vec::new();
            for i in 0..pass_count {
                let p = graph.add_pass(&format!("pass_{}", i));
                let res = format!("buf_{}", i);
                graph.set_pass_writes(p, &[&res]);
                if i > 0 {
                    let prev_res = format!("buf_{}", i - 1);
                    graph.set_pass_reads(p, &[&prev_res]);
                }
                passes.push(p);
            }
            graph.compile().unwrap();
            std::hint::black_box(graph.execution_order());
        });
    }
    println!();
}

// ─────────────────────────────────────────────────
// Transform Benchmark
// ─────────────────────────────────────────────────

fn bench_transform_matrix() {
    println!("━━━ Scene: Transform Matrix ━━━");

    let tf = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));

    bench("Transform::matrix()", 1_000_000, || {
        std::hint::black_box(std::hint::black_box(&tf).matrix());
    });

    let parent = Transform::from_translation(Vec3::new(10.0, 0.0, 0.0));
    let child = Transform::from_translation(Vec3::new(0.0, 5.0, 0.0));

    bench("Transform::mul_transform()", 1_000_000, || {
        std::hint::black_box(
            std::hint::black_box(&parent).mul_transform(std::hint::black_box(&child)),
        );
    });
    println!();
}
