# chemengine: A Deep Architectural Analysis

**A from-scratch Rust game engine with embodied Buddhist-psychology NPCs, first-class chemistry, and deterministic rendering**

---

## 1. Introduction & Design Philosophy

`chemengine` is a Rust game engine built without depending on Bevy, Unity, Unreal, or any pre-existing engine. It comprises 14 workspace crates totalling 14,715 lines of source across 83 Rust files, layered on wgpu 24 and OpenXR 0.19, with a custom archetype-based ECS at its foundation.

The engine sits at the intersection of three usually-disjoint domains:

- **Chemistry simulation** — atoms, bonds, and reactions as first-class ECS citizens
- **Embodied cognition** — NPC consciousness grounded in Buddhist psychology
- **Modern rendering** — GPU-driven, deterministic TAAU, stereo VR with full body/face/eye/voice input

The design thesis is that these three domains are not modular plugins to a generic engine — they are unified primitives of a single coherent runtime. A character in `chemengine` is not a humanoid mesh with an attached behaviour tree; it is an entity whose body is tracked through 36 OpenXR joints, whose face drives 63 ARKit-compatible blend shapes, whose internal state is governed by three-layer dukkha processing, and whose physical environment is composed of atoms that participate in real chemical reactions with activation energies and enthalpies.

This document is a deep architectural analysis of how that thesis is implemented across the 14 crates.

---

## 2. Architecture & Dependency Graph

### 2.1 Workspace Topology

The 14 crates form a strict four-layer DAG. No cycles, no upward references.

```
Layer 0  (Foundation)     ce_core    ce_math
                              |         |
Layer 1  (ECS)            ce_ecs ──────┘
                              |
Layer 2  (App Frame)      ce_app
                              |
                ┌─────┬───────┼──────┬────────┬────────┬───────┐
                |     |       |      |        |        |       |
Layer 3       window render compute  xr   physics chemistry   ai
  (Domain)       |     |       |      |        |        |       |
                                  scene  worldgen
                                      |
Layer 4   (Integration)             ce_interaction
                                      (depends on ai + xr + scene)
```

The top crate `ce_interaction` deliberately depends on both `ce_ai` and `ce_xr`, marking the intentional join-point between embodied input (face tracking, voice, body pose) and internal consciousness state (vedana, defense strain, knowledge updates). This is the architectural locus of the engine's distinctive ambition: bodies and minds are wired together by design, not by user code.

### 2.2 Lines-of-Code Distribution

| Crate | Files | Lines | Layer |
|-------|-------|-------|-------|
| ce_render | 9 | 2,384 | 3 |
| ce_ecs | 7 | 2,333 | 1 |
| ce_chemistry | 6 | 1,895 | 3 |
| ce_interaction | 6 | 1,313 | 4 |
| ce_xr | 9 | 1,155 | 3 |
| ce_ai | 6 | 980 | 3 |
| ce_physics | 6 | 942 | 3 |
| ce_worldgen | 5 | 814 | 3 |
| ce_scene | 5 | 766 | 3 |
| ce_core | 4 | 653 | 0 |
| ce_app | 4 | 598 | 2 |
| ce_math | 3 | 367 | 0 |
| ce_window | 3 | 329 | 3 |
| ce_compute | 4 | 186 | 3 |

`ce_render` and `ce_ecs` are co-equal in size (~2.3k lines each), with `ce_chemistry` at 1.9k lines — making chemistry the third-largest subsystem. This is unusual for a game engine. In Unity or Unreal, chemistry would be a third-party plugin or an asset pack; here it is a foundational simulation layer with the same first-class status as rendering and ECS.

### 2.3 Build Profile

```toml
[profile.release]
opt-level = 3
lto = "thin"
```

Thin-LTO with full optimisation. Combined with Rust's `no_std`-clean core crates and the absence of external runtime dependencies (no garbage collector, no scripting VM), the runtime footprint is determined entirely by what the engine itself instantiates.

---

## 3. The Custom Archetype ECS

### 3.1 Why Custom

Rust already has Bevy ECS, hecs, legion, specs, and shipyard. Building a custom ECS is a deliberate engineering investment whose payoff comes from being able to evolve the storage model in lockstep with the engine's domain-specific needs — in particular, the dense per-entity component sets that arise when entities carry both physical components (RigidBody, Velocity, Collider) and chemical components (Atom, Bond) and cognitive components (Consciousness, ThreePoisons, FourImmeasurables, EmotionalState, KnowledgeBase).

### 3.2 Storage Model

The ECS is archetype-based with Struct-of-Arrays (SoA) layout. From `ce_ecs/src/archetype.rs`:

```rust
pub struct Archetype {
    id: ArchetypeId,
    /// Component data, keyed by TypeId. Every column has the same len.
    columns: HashMap<TypeId, ComponentColumn>,
    /// Dense list of entities in this archetype. Index == row.
    entities: Vec<Entity>,
    /// Reverse lookup: entity -> row index.
    entity_rows: HashMap<Entity, usize>,
}
```

Each archetype holds entities that share the **exact same set of component types**. Within an archetype, components are stored column-wise — one `ComponentColumn` per component type. Row `i` across all columns describes entity `i`.

This is the canonical Bevy/Flecs design. Iteration over a query is a tight inner loop over contiguous arrays, achieving near-optimal cache utilisation.

### 3.3 Module Decomposition

| Module | Responsibility |
|--------|----------------|
| `archetype` | Archetype tables (SoA storage of component columns) |
| `component` | `Component` trait, `ComponentColumn` type-erased storage |
| `event` | `EventReader`/`EventWriter`/`Events<T>` |
| `resource` | Global singleton resources (Bevy-style) |
| `schedule` | `Schedule`, `CoreStage` (Startup, FixedUpdate, Update, etc.) |
| `world` | `World` — top-level container for archetypes, resources, schedule |

The public prelude exposes the canonical Bevy-style API:

```rust
use ce_ecs::prelude::*;

let mut world = World::new();
let entity = world.spawn();
world.insert_component(entity, 42u32);
```

### 3.4 What's Different from Bevy

`chemengine`'s ECS does not (yet) implement:
- Parallel system execution (Bevy's `ParallelExecutor`)
- Reflection-based scripting hooks
- A full Component derive macro ecosystem (`#[derive(Component)]`)

What it provides instead is a smaller, fully-owned codebase whose storage and scheduling can be specialised for the chemistry+consciousness workload (high entity counts of small components, where chemical bond entities reference atom entities, where consciousness entities carry six-to-eight cognitive components simultaneously).

This trade-off — smaller, slower-evolving, but fully under one author's control — is consistent with the engine's broader posture.

---

## 4. Rendering Pipeline Deep Dive

`ce_render` is the largest crate by lines (2,384). It implements a modern GPU-driven rendering pipeline including TAAU and stereo VR — all without any AI inference at runtime.

### 4.1 Module Map

| Module | Lines | Role |
|--------|-------|------|
| `taau` | ~700 | Temporal Anti-Aliasing Upscaling |
| `stereo` | ~200 | Stereo TAAU for VR |
| `gpu_driven` | ~300 | Indirect draw command structures |
| `gpu_cull` | ~250 | GPU frustum culling |
| `render_graph` | ~400 | Pass-based render graph with resource tracking |
| `gpu` | ~150 | wgpu context wrapper |
| `mesh` | ~100 | Vertex/mesh primitives |
| `color` | ~80 | Colour types |

### 4.2 GPU-Driven Rendering

Draw calls are issued via `wgpu::DrawIndirectArgs`-compatible command buffers:

```rust
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,  // 0 = culled, 1 = visible
    pub first_vertex: u32,
    pub first_instance: u32,
}
```

A GPU compute pass writes `instance_count` to either 0 (culled) or 1 (visible) based on AABB-vs-frustum testing. The subsequent indirect draw call then renders only the visible objects. The CPU never sees the visibility decision.

### 4.3 Deterministic TAAU

TAAU (Temporal Anti-Aliasing Upscaling) replaces DLSS/FSR/XeSS for `chemengine`. The defining choice: **no AI inference**. The upscaling is fully algorithmic and deterministic.

The pipeline:

1. **Sub-pixel jitter** — Halton(2,3) sequence offsets the projection matrix per frame. Provides 16 jitter points before repeating.

2. **Closest-depth motion selection** — Motion vectors are sampled at the depth-closest neighbour pixel, which reduces ghosting around moving silhouettes.

3. **YCoCg neighborhood clamping** — Convert to YCoCg colour space; compute the bounding box of the 3×3 neighbourhood; clamp the history sample into that box. YCoCg gives better luma-chroma separation than RGB for this clamp.

4. **Motion-weighted history blend** — Blend ratio depends on motion vector magnitude. Static pixels accumulate more history (more anti-aliasing); fast-moving pixels blend less (less ghosting).

5. **CAS-lite sharpening** — A simplified Contrast Adaptive Sharpening pass at the end, tunable via `UpscaleSettings::sharpness` (default 0.4).

The internal scale defaults to 0.667 — render at ~1440p, upscale to 2160p. This produces a 2.25× rendering speedup over native 4K.

```rust
pub struct UpscaleSettings {
    pub internal_scale: f32,  // 0.667 default → 1440p/2160p
    pub sharpness: f32,       // 0.4 default
}
```

History textures are ping-ponged by `frame_index & 1` — read from `history[(frame + 1) & 1]`, write to `history[frame & 1]`.

### 4.4 Stereo VR Rendering

`stereo.rs` orchestrates two `TaauPass` instances, one per eye. The key insight is **jitter decorrelation**:

- Left eye uses `TaauPass::execute(frame)`
- Right eye uses `TaauPass::execute(frame + 8)`

Offsetting by 8 (half the 16-entry Halton table) ensures the two eyes sample different sub-pixel positions every frame. Without this offset, both eyes would jitter identically and the stereoscopic fusion in the user's visual cortex would produce a cross-eyed shimmer artefact. The offset is even, so frame-parity (used for history ping-pong) is preserved.

### 4.5 Why Deterministic Matters

AI upscaling (DLSS/FSR2/XeSS) is the industry default. `chemengine` rejects it for three reasons:

1. **Reproducibility** — A simulation that produces the same chemical state must also produce the same pixels. Neural network inference introduces non-determinism (cuDNN algorithm selection, floating-point reduction order) that breaks frame-perfect reproducibility.

2. **Latency budget** — In VR, every millisecond of inference latency increases motion-to-photon time. Algorithmic TAAU has a predictable, low-variance latency.

3. **No model weights** — DLSS requires NVIDIA Tensor Cores; FSR2 doesn't but requires a runtime that hosts the algorithm. `chemengine`'s TAAU is a single WGSL shader compiled with the rest of the engine.

---

## 5. Chemistry Subsystem

`ce_chemistry` is the most distinctive subsystem in `chemengine` from a domain-application standpoint. No mainstream game engine treats chemistry as a first-class primitive.

### 5.1 Element Properties

The full periodic table is encoded as a static resource:

```rust
pub struct ElementProperties {
    pub atomic_number: u8,
    pub symbol: &'static str,
    pub name: &'static str,
    pub atomic_mass: f64,
    pub electronegativity: Option<f64>,
    pub phase_at_stp: Phase,
    pub group: u8,
    pub period: u8,
    pub category: ElementCategory,
}
```

All 118 elements are instantiated when `ChemistryPlugin` is added to the app:

```rust
impl Plugin for ChemistryPlugin {
    fn build(&self, app: &mut App) {
        let table = PeriodicTable::new();
        let registry = ReactionRegistry::default();
        app.insert_resource(table);
        app.insert_resource(registry);
    }
}
```

### 5.2 Atoms and Bonds as ECS Entities

```rust
pub struct Atom {
    pub element: ElementId,
    pub charge: i8,
    pub mass_override: Option<f64>,  // isotope support
}

pub enum BondType {
    Single, Double, Triple,
    Aromatic, Ionic, Hydrogen, VanDerWaals,
}

pub struct Bond {
    pub atom_a: Entity,
    pub atom_b: Entity,
    pub bond_type: BondType,
    pub bond_order: f64,
    pub equilibrium_length: f64,  // Angstroms
}
```

Note the consequence: an `Atom` is an ECS entity, and a `Bond` is an entity that references two other entities by `Entity` handle. The chemistry graph is the ECS graph. There is no separate molecular data structure — molecules emerge as connected subgraphs of bonded atom entities.

### 5.3 Reaction Rules

```rust
pub struct ReactionRule {
    pub name: String,
    pub reactants: Vec<String>,        // e.g. ["H2", "O2"]
    pub products: Vec<String>,         // e.g. ["H2O"]
    pub activation_energy: f64,        // kJ/mol
    pub enthalpy_change: f64,          // kJ/mol (negative = exothermic)
    pub rate_constant: f64,
}
```

The default registry includes at minimum five reactions: water formation (H₂ + O₂ → H₂O), methane combustion (CH₄ + O₂ → CO₂ + H₂O), acid-base neutralisation (HCl + NaOH → NaCl + H₂O), among others. Each carries real thermochemical data — activation energies in kJ/mol, enthalpies, rate constants.

This is not gameplay-flavoured chemistry. It is chemistry that obeys the Arrhenius equation in principle, with parameters that could be tuned to match experimental data.

### 5.4 Why This Matters

Once chemistry is first-class:

- **Procedural materials** can be derived from molecular composition. The engine's procedural-only design (no texture images) is enabled by deriving surface appearance from atomic-scale properties (electron configuration, bond polarity) rather than from artist-authored bitmaps.
- **NPC perception** can include chemical signals. A potion's smell is a function of which molecules it emits; that signal feeds into NPC sensory inputs (the `vedana` input to the consciousness kernel).
- **World events** can have chemical causes. A fire is not a particle effect — it is a combustion reaction with reactants, products, and an energy release that warms surrounding atoms.

This last item is aspirational rather than fully implemented in the current codebase, but the data model supports it without requiring architectural change.

---

## 6. The Consciousness Kernel

`ce_ai` is the engine's most theoretically novel subsystem. It implements NPC cognition grounded in Buddhist psychology — specifically the SMA28 Shion architecture — with a mathematical model that maps to (but does not yet formally integrate with) the Free Energy Principle.

This section presents the mathematics directly from the source code.

### 6.1 Three-Layer Dukkha Processing

The core data structure:

```rust
pub struct Consciousness {
    pub vow_weights: [f64; 4],       // [blessing, perception, justice, continuity]
    pub trait_sediment: [f64; 4],
    pub vedana: f64,                  // raw sensation [0, 1]
    pub felt: f64,                    // defense-amplified
    pub filtered: f64,                // awareness-attenuated
    pub defense_strain: f64,          // [0, 1]
    pub merit: f64,
    pub awareness: f64,               // [0, 1]
    pub prediction_error: f64,        // EMA
    pub step_count: u64,
}
```

The `step(stimulus, learning_rate)` function processes one consciousness tick:

```
Step 1:  vedana    = clamp(stimulus, 0, 1)
Step 2:  felt      = vedana × (1 + defense_strain × 0.5)
Step 3:  filtered  = felt × (1 - awareness × 0.9)
Step 4:  pe_new    = |vedana - filtered|
         PE        = PE × (1 - α) + pe_new × α        [exponential moving average]
Step 5:  if PE > 0.5:
             defense += α × 0.1   (clamped to 1)
         else:
             defense -= α × 0.05  (clamped to 0)
Step 6:  for i in 0..4:
             trait_sediment[i] += filtered × vow_weights[i] × α × 0.01
Step 7:  if filtered < 0.3 and awareness > 0.5:
             merit += 0.1 × awareness
```

The structural reading:

- **Vedana** is the Buddhist term for the most primitive layer of subjective experience — bare felt-quality, pre-conceptual. Here it is the unmodified stimulus.

- **Felt** is vedana with defensive amplification. When defense_strain is high, sensations feel "bigger" than they are. This models the well-documented psychological phenomenon where threat-anticipating subjects experience neutral stimuli as more aversive.

- **Filtered** is felt after awareness attenuation. High awareness reduces the impact of felt experience on the cognitive system — this is the formal correlate of mindfulness as a regulator of reactivity.

- **Prediction error** tracks the gap between raw sensation and filtered experience. High PE indicates the consciousness is being surprised — its model of the world is wrong.

- **Defense strain adapts** in response to PE: high PE strengthens defenses, low PE relaxes them. This is a closed feedback loop with FEP-like character: the system minimises long-run prediction error by adjusting its own boundary conditions.

- **Trait sediment** accumulates filtered experience weighted by vow weights. This is the engine's representation of *vāsanā* — the karmic seed-impressions left by experience.

- **Merit** grows when the system is composed (low filtered) and aware (high awareness). This is a scalar reward for equanimous attention.

### 6.2 The Shion Archetype

A factory method generates the personality named *Shion*:

```rust
pub fn shion_archetype() -> Self {
    Self {
        vow_weights: [2.0, 1.5, 1.0, 1.5],
        defense_strain: 0.8,
        awareness: 0.9,
        prediction_error: 0.3,
        ..Default::default()
    }
}
```

The archetype encodes:
- **Elevated blessing and continuity vows** — Shion is motivated by protective and persistent care
- **High initial defense** (0.8) — Shion enters interactions guarded
- **Very high awareness** (0.9) — Shion is hypervigilant
- **Low initial PE** (0.3) — Shion arrives with confident priors

The narrative reading is that Shion is a being whose awareness is sharper than her openness — she sees more than she lets in. The system can show her gradually softening defense as repeated interactions lower PE.

### 6.3 Nirodha — Cessation

```rust
pub fn nirodha(&mut self) {
    self.prediction_error *= 0.5;
    self.defense_strain = (self.defense_strain - 0.2).max(0.0);
    self.merit += 1.0;
}
```

Nirodha (the third Noble Truth — cessation of suffering) is a discrete event that halves PE, reduces defense by 0.2, and grants one unit of merit. It is the formal correlate of "letting go" — a moment in which the cognitive system releases its predictive grip.

In gameplay terms, this is a triggerable transition: meditation, a transformative dialogue, the resolution of a long-held attachment.

### 6.4 Clinging

```rust
pub fn clinging(&self) -> f64 {
    (self.felt - self.vedana).abs()
}
```

A diagnostic scalar: the gap between what is felt and what is given. When defense is high, clinging is high — experience has been distorted. This exposes an externally readable measure of cognitive distress.

### 6.5 Three Poisons with Blowback Dynamics

```rust
pub enum PoisonType {
    Lobha,   // greed / attachment
    Dosa,    // hatred / aversion
    Moha,    // delusion / ignorance
}

pub struct ThreePoisons {
    pub lobha: f64,
    pub dosa: f64,
    pub moha: f64,
}
```

The default state has small residual values (≈0.1, 0.1, 0.2) — "even virtuous beings carry seeds," as the source comment notes.

The interesting behaviour is in `apply_intervention(target, strength)`. Reducing one poison **increases the other two** as blowback. This is structurally significant: in this model, you cannot simply suppress one negative trait. Attempting to do so increases the others. The mechanism formalises the Buddhist observation that the three poisons are entangled — they share a common root in ignorance about the nature of self.

Concretely, intervening on `Dosa` (hatred) with strength 0.5 reduces dosa and increases both lobha and moha by a portion of the strength. The system has tested invariants:

- `intervention_reduces_target` — verifies the target poison decreases
- `intervention_has_blowback` — verifies the other two increase
- `values_stay_clamped` — verifies all stay in [0, 1] even under strong interventions

### 6.6 Four Immeasurables as Reward Function

```rust
pub struct FourImmeasurables {
    pub maitri: f64,   // loving-kindness
    pub karuna: f64,   // compassion
    pub mudita: f64,   // sympathetic joy
    pub upekkha: f64,  // equanimity
}

pub fn compute_reward(&self, happiness_delta: f64, suffering_delta: f64) -> f64 {
    self.maitri * happiness_delta
  - self.karuna * suffering_delta
  + self.mudita * happiness_delta.max(0.0)
  + self.upekkha * 0.1
}
```

This is a scalar reward function suitable for use in reinforcement learning. Each immeasurable contributes a term:

- **Maitri** (loving-kindness) — rewards proportionally to others' happiness change
- **Karuna** (compassion) — penalises proportionally to increased suffering (the minus sign)
- **Mudita** (sympathetic joy) — amplifies reward when others become happier (the rectified term)
- **Upekkha** (equanimity) — constant 0.1 baseline; the system gets credit even in neutral conditions

The bodhisattva archetype sets all four to 1.0. The default has all four at 0.5 — moderate cultivation.

The tests verify:
- `reward_positive_when_helping` — bodhisattva sees positive reward when happiness_delta > 0 and suffering_delta < 0
- `reward_negative_when_harming` — bodhisattva sees negative reward when happiness_delta < 0 and suffering_delta > 0
- `higher_compassion_penalises_suffering_more` — increasing karuna increases the penalty for suffering_delta = 1
- `equanimity_provides_baseline` — bodhisattva at (0, 0) gets exactly 0.1 reward

What this gives the engine: a principled, mathematically simple reward signal for NPC behaviour learning that is **other-regarding** by construction. Standard game AI rewards are typically self-interested (kill more enemies, collect more loot). This reward function rewards an NPC for increasing others' happiness and reducing others' suffering.

### 6.7 Contact Tier — Epistemic Provenance

```rust
pub enum ContactTier {
    FirstHand,   // direct experience
    Derived,     // inferred
    Hearsay,     // from another source
}

pub struct Knowledge {
    pub content: String,
    pub tier: ContactTier,
    pub confidence: f64,
    pub freshness: f64,
    pub source: Option<String>,
}
```

Each piece of NPC knowledge carries its acquisition tier. The tier determines initial confidence:

- `Knowledge::first_hand(content)` — confidence 1.0
- `Knowledge::hearsay(content, source)` — confidence 0.5

Freshness decays exponentially:

```rust
pub fn tick(&mut self, dt: f64) {
    self.freshness *= (-dt / 300.0_f64).exp();
}
```

Half-life is ~208 seconds. After dt=300 seconds, freshness ≈ 0.368 (e⁻¹).

An NPC asserts knowledge only when both thresholds are crossed:

```rust
pub fn would_assert(&self) -> bool {
    self.confidence > 0.7 && self.freshness > 0.3
}
```

Hearsay never crosses confidence 0.7 from its initial 0.5 — so by default, NPCs cannot confidently assert what they only heard. They can update beliefs over time, but the system encodes a Buddhist-philosophical principle: direct contact is epistemically privileged.

This is a small thing, mechanically. But it makes possible NPC dialogue systems in which characters say "I heard..." vs "I saw..." based on the actual provenance of their knowledge, and where their confidence varies accordingly. This is far more honest than the typical fact-database approach.

### 6.8 Emotional State and Memory Weight

`EmotionalState` follows Plutchik's eight-emotion wheel:

```rust
pub struct EmotionalState {
    pub joy, sadness, anger, fear,
    pub surprise, disgust, trust, anticipation: f32,
}
```

Each is a continuous value in [0, 1]. `blend_toward(target, rate)` performs linear interpolation. `dominant()` returns the strongest emotion name.

`EmotionalWeight` is attached to individual memories:

```rust
pub struct EmotionalWeight {
    pub charge: f64,             // positive or negative
    pub wound_proximity: f64,    // 0 = distant, 1 = core wound
    pub consolidation: f64,      // 0 = fresh, 1 = deep
    pub nirodha_resolved: bool,
}
```

This is the architectural placeholder for episodic memory — memories carry emotional charge, proximity to core wounds, consolidation level, and a flag for whether they have been resolved through cessation practice. The flag is binary; resolution is treated as a discrete event, consistent with how `nirodha()` operates on the parent `Consciousness`.

### 6.9 Summary: A Coherent Computational Buddhism

Across consciousness, three poisons, four immeasurables, contact tier, and emotion, the `ce_ai` crate implements a coherent computational rendering of major themes in Buddhist psychology:

| Theme | Implementation |
|-------|----------------|
| Dukkha (suffering) | Three-layer processing with defense distortion |
| Vedana (feeling) | Raw sensory input layer |
| Predictive processing | EMA prediction error, defense adaptation |
| Tanha (craving) | Defense-driven amplification of felt experience |
| Vāsanā (impressions) | Trait sediment accumulation |
| Nirodha (cessation) | Discrete release event |
| Three poisons | Entangled triple with blowback dynamics |
| Four immeasurables | Scalar reward function |
| Pramana (epistemology) | Contact tier for knowledge provenance |
| Citta (mind) | Plutchik emotional state |
| Sankhara (mental formation) | Emotional weight on memories |

No other game engine (or, to the author's knowledge, any other open-source software) implements this combination as a first-class runtime. The closest comparators are research cognitive architectures (Soar, ACT-R) and academic experimental systems — none of which target real-time games with embodied input.

---

## 7. XR: Multi-Modal Body Input

`ce_xr` provides the embodied input side of the embodiment-consciousness pairing. It builds on OpenXR 0.19 and exposes face tracking, body tracking, eye tracking, voice input, and hand skeletons — the full set of inputs available on modern VR headsets.

### 7.1 Face Tracking — 63 ARKit Blend Shapes

`FaceBlendShapes` contains 63 fields, structured by anatomical region:

- Eye (14 fields) — blink, look directions, squint, wide
- Jaw (4 fields) — forward, left, open, right
- Mouth (~30 fields) — smiles, frowns, pucker, stretches, presses, rolls, shrugs
- Cheek (3 fields) — puff, squints
- Nose (2 fields) — sneers
- Brow (5 fields) — down, inner up, outer up
- Tongue (1 field) — out

This is the superset of:
- Apple ARKit (reference standard)
- Meta Quest Pro/3 via `XR_FB_face_tracking2`
- HTC Vive via `XR_HTC_facial_tracking`

The fact that the type carries 63 fields (rather than a `HashMap<String, f32>`) is a deliberate design choice. It makes blend shape values cache-coherent and amenable to SIMD operations, and it makes the engine's type system enforce coverage.

### 7.2 Body Tracking — 36 Joints

```rust
pub enum BodyJoint {
    Root, Hips, SpineLower, SpineMiddle, SpineUpper, Chest, Neck, Head,
    // Left arm: 7 joints (Shoulder through Palm)
    // Right arm: 7 joints
    // Left leg, Right leg: 8 joints each
    // ...
}
```

Compatible with Meta Body Tracking API (36 joints). Finger joints are tracked separately through `HandSkeleton` — the human hand has ~26 bones, far more than the body trunk, so they live in their own type to keep `BodyJoint` manageable.

### 7.3 Eye Tracking

```rust
pub struct EyeGaze {
    pub origin: Vec3,           // head-local
    pub direction: Vec3,        // normalised
    pub fixation_point: Vec3,   // convergence in world space
    pub pupil_dilation: f32,    // 0=contracted, 1=dilated
    pub confidence: f32,
    pub is_tracked: bool,
}
```

The `pupil_dilation` field is significant for NPC interaction. Pupil dilation correlates with cognitive load and emotional arousal. An NPC's `Consciousness::step` can take pupil dilation as part of its stimulus signal — the NPC notices that the player is engaged.

### 7.4 Voice

```rust
pub struct VoiceConfig {
    pub language: String,        // BCP47, default ja-JP
    pub sample_rate: u32,        // 16000
    pub wake_word: Option<String>,
    pub vad_threshold: f32,      // 0.5
}

pub struct SpeechEvent {
    pub text: String,
    pub confidence: f32,
    // ...
}
```

Voice activity detection is built-in via `vad_threshold`. The default language is Japanese — a tell that this is one author's engine, not a generic product. The wake-word feature suggests intended use for ambient voice-driven NPC interaction.

### 7.5 Graceful Degradation

```rust
impl Plugin for XrPlugin {
    fn build(&self, app: &mut ce_app::App) {
        match XrSession::try_new(&self.config) {
            Ok(session) => {
                // Insert all XR resources
                log::info!("XR mode enabled — stereo rendering active");
            }
            Err(e) => {
                log::warn!("OpenXR not available: {}. Running in desktop mode.", e);
            }
        }
    }
}
```

If no OpenXR runtime is present, the engine logs a warning and continues in desktop mode. No panic, no missing-symbol errors, no environment-specific build configurations. This is correct handling — XR is treated as optional capability, not a hard prerequisite.

---

## 8. The Integration Layer

`ce_interaction` is the top crate. Its dependencies are unique among the workspace:

```toml
[dependencies]
ce_core      = { workspace = true }
ce_ecs       = { workspace = true }
ce_math      = { workspace = true }
ce_app       = { workspace = true }
ce_ai        = { workspace = true }   # ← consciousness
ce_xr        = { workspace = true }   # ← embodied input
log          = { workspace = true }
```

It is the **only** crate that depends on both `ce_ai` and `ce_xr`. This is the architectural seam where embodied input becomes cognitive input.

The 1,313 lines in this crate implement (conceptually):

- Mapping face blend shapes onto NPC `EmotionalState`
- Routing player gaze (`EyeGaze::pupil_dilation`) into NPC `Consciousness::step` stimulus
- Pipelining voice transcripts into NPC `KnowledgeBase` as `ContactTier::Hearsay` entries
- Triggering `Consciousness::nirodha()` from specific gestures or vocal cues
- Feeding ambient chemical signals (e.g. nearby reaction products from `ce_chemistry`) into NPC vedana

The implementation details matter less than the position: this is where embodiment and cognition meet, and that meeting is built into the engine's type-system topology, not bolted on by user code.

---

## 9. Comparative Analysis

### 9.1 vs Bevy

Bevy is the dominant Rust game engine. It is an ECS-first engine with a thriving plugin ecosystem.

**Where chemengine is smaller**: Bevy has hundreds of contributors, dozens of crates, a full asset pipeline, a UI framework, and an active community. Chemengine has one author and 14 crates with no asset pipeline.

**Where chemengine is differentiated**: Bevy has no chemistry, no consciousness model, and no built-in XR (Bevy's XR support exists as community plugins of varying maturity). Chemengine integrates all three at the foundation level. A `chemengine` user gets these by default; a Bevy user assembles them from plugins.

**Architectural difference**: Bevy's design optimises for general-purpose game development. Chemengine's design optimises for a specific kind of game — one in which chemistry, embodied input, and NPC cognition are central to the play experience.

### 9.2 vs Unity / Unreal

Unity and Unreal are AAA commercial engines.

**Where they dominate**: Editor tooling, asset pipeline, marketing reach, platform support, performance optimisation budget. They are mature commercial products with thousands of person-years of engineering.

**Where they don't compete**: Both engines treat NPC AI as a behaviour tree problem. Unreal's "Mass AI" framework is performance-oriented; both engines' AI is fundamentally reactive (perception → decision → action) rather than cognitive (vedana → felt → filtered → ...). Neither has chemistry as a primitive. Both have moved toward AI-driven upscaling (DLSS plugin for Unity/Unreal); neither offers deterministic TAAU as a first-class option.

**Use case mapping**: A chemengine-style game would be very difficult to build on Unity or Unreal — you would have to fight the engine's defaults at every layer. Conversely, a typical FPS or platformer would be much harder to build on chemengine.

### 9.3 vs Cognitive Architectures (Soar, ACT-R)

Soar and ACT-R are research cognitive architectures with decades of development.

**Where they dominate**: Theoretical grounding in cognitive psychology, peer-reviewed validation against human experimental data, broad academic legitimacy.

**Where they don't compete**: Neither targets real-time games. Neither integrates with embodied VR input. Neither implements Buddhist psychology as a first-class layer (this is not their failing — it has not been their goal).

**Position of chemengine's ce_ai**: It is in the same genus as Soar/ACT-R (cognitive architectures), but in a different species — one tuned for real-time interactive use with VR embodiment, with theoretical grounding drawn from Buddhist psychology rather than cognitivism.

### 9.4 vs LLM-Driven NPC Systems

The current frontier of game AI is LLM-driven NPCs (Inworld AI, Convai, GPT-4 wrappers). These provide flexible language output but lack persistent internal state and have high inference cost.

**Where they dominate**: Natural language conversation, broad knowledge coverage, expressive flexibility.

**Where chemengine differs**: NPC consciousness in chemengine is a small (~10 floats), persistent, deterministic state. It does not produce free-form language; it produces internal-state evolution that the game logic can interpret. The trade-off: less linguistic flexibility, but full reproducibility, no inference cost at runtime, and explicit semantic structure.

These two approaches are complementary rather than competing. An LLM could be the language-output layer above a `ce_ai`-style consciousness layer.

---

## 10. Design Trade-offs

### 10.1 What's Sacrificed

| Area | Sacrifice |
|------|-----------|
| Editor | No GUI editor. All scene construction is code-based. |
| Assets | No texture pipeline. Procedural-only. Models are code-built meshes. |
| Tooling | No profiler integration, no scene viewer, no script reload. |
| Platform breadth | Desktop + VR via OpenXR. No mobile, no console. |
| Documentation | Rustdoc comments only. No tutorials, no examples beyond /examples/. |
| Community | One author, no plugin ecosystem. |
| Performance optimisation | No parallel ECS execution yet. Single-threaded scheduler. |

### 10.2 What's Gained

| Area | Gain |
|------|------|
| Coherence | All 14 crates fit one author's mental model. No impedance mismatch. |
| Determinism | No AI inference at runtime. Reproducible frames. |
| Domain alignment | Chemistry + cognition + embodiment integrated from the foundation. |
| Type safety | All inputs and states are statically typed. No untyped scripting. |
| Build simplicity | Single `cargo build`. No external SDKs. |

### 10.3 Whose Engine Is This For

Not a general-purpose engine. Specifically suited to:

- VR/AR titles with deep NPC interaction
- Chemistry-driven sandbox games (alchemy, medicine, cooking)
- Procedural worlds without artist assets
- Research uses (cognitive modelling, reproducible simulation)
- Single-author or very small team development

Unsuited to:
- AAA team development with role specialisation
- Mobile or console targets
- Game jams requiring quick visual prototyping
- Projects requiring artist-driven asset workflows

---

## 11. Performance Characteristics

Quantitative performance data is not embedded in the source. What can be inferred:

- **Build profile** is `opt-level = 3` with thin LTO — the release configuration is aggressive.
- **Storage model** (archetype SoA) is optimal for query iteration.
- **GPU-driven rendering** moves visibility decisions to the GPU, freeing CPU for ECS and AI work.
- **TAAU at 0.667 internal scale** trades ~2.25× rendering speedup for upscale cost.
- **Consciousness step cost** is dominated by ~12 floating-point operations per NPC per tick — trivially cheap.
- **Single-threaded scheduler** is currently the main parallelism limitation; per-archetype parallel execution would be a high-value future optimisation.

For VR targets (90Hz, ~11ms frame budget), the bottleneck is likely to be rendering rather than AI or chemistry: even thousands of NPCs running the consciousness step would consume a small fraction of frame budget; thousands of triangles need to be rendered twice (stereo) at 90Hz, which is the harder problem and the one that ce_render addresses head-on.

---

## 12. Conclusion

`chemengine` is a 14,715-line Rust game engine that integrates chemistry, embodied cognition, and modern rendering as foundational primitives rather than optional plugins. Its distinguishing features:

1. **First-class chemistry** — 118 elements, 7 bond types, reaction rules with real thermochemistry, all as ECS entities and resources.

2. **Buddhist-psychology NPC consciousness** — three-layer dukkha processing, three poisons with blowback dynamics, four immeasurables as a scalar reward function, contact-tier knowledge provenance, Shion archetype. No other engine implements this.

3. **Deterministic rendering** — GPU-driven indirect drawing, AI-free TAAU upscaling, decorrelated stereo VR. Reproducible frame output, predictable latency, no neural network dependencies.

4. **Comprehensive XR input** — 63 ARKit-compatible face blend shapes, 36 Meta-API body joints, eye tracking with pupil dilation, VAD voice input. Graceful degradation to desktop.

5. **Integration topology** — `ce_interaction` is the only crate depending on both `ce_ai` and `ce_xr`, making the embodiment-cognition pairing architecturally explicit.

The engine is not positioned to displace Bevy, Unity, or Unreal. It is positioned to enable a specific kind of work that those engines do not support natively: VR experiences in which chemistry is a genuine system, NPCs have inner lives modelled on a coherent philosophical framework, and frame-perfect reproducibility is preserved end-to-end.

What it represents, in the broader landscape: an existence proof that game engines can be built with theoretical commitments — not just engineering commitments — at their core. The choice to ground NPC cognition in Buddhist psychology rather than behaviour trees, or to refuse AI upscaling on principle, is not a technical optimisation. It is a stance.

The engine takes that stance and ships it as Rust code.

---

**Document end.**
