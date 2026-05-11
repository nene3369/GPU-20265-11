[chemengine_analysis.md](https://github.com/user-attachments/files/27591436/chemengine_analysis.md)
# chemengine 全体構造分析

**自作 Rust ゲームエンジンの技術文書**

---

## 0. 概要

chemengine は Rust で書かれた自作ゲームエンジン。Bevy / Unity / Unreal とは別系統で、wgpu 24、OpenXR 0.19、自作 archetype ECS を基盤とする。

### 規模数値

| 項目 | 値 |
|------|-----|
| crate 数 | 14 (workspace member) |
| examples 数 | 6 |
| Rust ソースファイル数 | 83 (crates 内) |
| 総コード行数 | 14,715 行 |
| Edition | 2021 |
| 主要依存 | glam 0.29 / wgpu 24 / winit 0.30 / OpenXR 0.19 |

### 設計方針

外部エンジン (Bevy/Unity/Unreal) に乗らず、ECS / レンダリング / 物理 / 化学 / NPC AI を全て自作する。これにより:

- 化学シミュレーションがエンジン基盤に直結する
- NPC AI が独自の意識モデルで動く
- ゲームエンジンの枠を超えた構造を1スタックに統合する

---

## 1. crate 依存構造

14 crates は依存層で4階層になっている。

```
Layer 0 (基盤):     ce_core, ce_math
Layer 1 (ECS):      ce_ecs
Layer 2 (App):      ce_app
Layer 3 (Domain):   ce_window, ce_render, ce_compute, ce_xr,
                    ce_physics, ce_chemistry, ce_ai,
                    ce_worldgen, ce_scene
Layer 4 (統合):     ce_interaction
```

ce_interaction が ce_ai + ce_xr 両方に依存する最上層になっている。これは「身体性 (XR) と意識 (AI) の接続点」として機能する設計になっている。

---

## 2. crate 別行数ランキング

| 順位 | crate | files | lines | 役割 |
|------|-------|-------|-------|------|
| 1 | ce_render | 9 | 2,384 | レンダリングパイプライン |
| 2 | ce_ecs | 7 | 2,333 | Entity Component System |
| 3 | ce_chemistry | 6 | 1,895 | 化学シミュレーション |
| 4 | ce_interaction | 6 | 1,313 | XR + AI 統合層 |
| 5 | ce_xr | 9 | 1,155 | VR/AR (OpenXR) |
| 6 | ce_ai | 6 | 980 | NPC 意識カーネル |
| 7 | ce_physics | 6 | 942 | 物理シミュレーション |
| 8 | ce_worldgen | 5 | 814 | 手続き世界生成 |
| 9 | ce_scene | 5 | 766 | シーン管理 |
| 10 | ce_core | 4 | 653 | 基盤型 |
| 11 | ce_app | 4 | 598 | App / Plugin システム |
| 12 | ce_math | 3 | 367 | 数学ユーティリティ |
| 13 | ce_window | 3 | 329 | ウィンドウ (winit ラッパ) |
| 14 | ce_compute | 4 | 186 | GPU compute |

ce_render と ce_ecs がほぼ同じ規模で2強、その次に ce_chemistry が来るのが特徴。一般的なゲームエンジンと比べ、化学シミュ層が ECS と同じオーダーで実装されている。

---

## 3. crate 別詳細分析

### 3.1 ce_core — 基盤型

**役割**: Entity ID, SparseSet, ComponentTypeId の基盤定義。

**主要型**:
- `Entity`: エンティティハンドル
- `Entities`: エンティティ管理
- `SparseSet<T>`: スパース配列 (ECS の内部データ構造)
- `ComponentTypeId`: コンポーネント型 ID

**依存**: なし (最下層)

**設計意図**: ECS の基盤型を独立 crate として切り出し、他層からは ce_core のみ依存することで型の循環参照を避けている。

---

### 3.2 ce_math — 数学

**役割**: glam ラッパおよび独自数学ユーティリティ。

**依存**: glam 0.29

**設計意図**: glam を再エクスポートする薄い層。エンジン全体で同じ Vec3 / Quat 型を使うための単一エントリポイント。

---

### 3.3 ce_ecs — Entity Component System

**役割**: Archetype-based ECS。Struct of Arrays (SoA) レイアウト、cache-friendly。

**主要モジュール**:
- `archetype`: アーキタイプ管理 (同じコンポーネント集合を持つエンティティを1テーブルに格納)
- `component`: コンポーネント基盤
- `event`: EventReader/EventWriter (Bevy 風イベント)
- `resource`: グローバルリソース管理
- `schedule`: システムスケジューラ (CoreStage 段階制御)
- `world`: World 本体

**主要型**:
```rust
pub struct World { ... }
pub struct Archetype {
    id: ArchetypeId,
    columns: HashMap<TypeId, ComponentColumn>,
    entities: Vec<Entity>,
    entity_rows: HashMap<Entity, usize>,
}
pub enum CoreStage { ... }
```

**設計意図**: Bevy ECS と類似アーキテクチャだが完全自作。アーキタイプベースで、同じコンポーネント集合のエンティティをまとめて格納することで、クエリ時のキャッシュ効率を上げる。

---

### 3.4 ce_app — アプリケーションフレームワーク

**役割**: `App` ビルダー、`Plugin` トレイト、`Time` / `FixedTime` リソース。

**設計意図**: ECS の World と Schedule を所有し、Plugin を登録するためのアプリケーション基盤。ウィンドウイベントループは持たない (ce_window が担当)。

**主要 API**:
```rust
let mut app = App::new();
app.add_plugin(ChemistryPlugin)
   .add_plugin(AiPlugin)
   .add_plugin(PhysicsPlugin::default())
   .run();
```

---

### 3.5 ce_window — ウィンドウ管理

**役割**: winit ラッパ。プラットフォーム抽象化。

**依存**: winit 0.30

**設計意図**: ce_render から winit 直接依存を切り離し、ウィンドウ管理を ce_window に集約する。

---

### 3.6 ce_render — レンダリングパイプライン

**役割**: wgpu ベースの GPU レンダリング。GPU-driven、TAAU、Stereo VR まで含む。

**主要モジュール**:
- `color`: 色型
- `gpu`: GpuContext (wgpu 抽象化)
- `gpu_cull`: GPU フラスタムカリング
- `gpu_driven`: 間接描画コマンド (DrawIndirect)
- `mesh`: メッシュ / 頂点定義
- `render_graph`: リソース追跡型レンダーグラフ
- `stereo`: VR ステレオ TAAU (両眼レンダリング)
- `taau`: Temporal Anti-Aliasing Upscaling (AI フリー)

**TAAU の特徴**:
- 決定論的、AI 推論を一切使わない
- Halton(2,3) ジッタリング
- Closest-depth モーションベクトル選択
- YCoCg 近傍クランピング
- モーション重み付き履歴ブレンド
- CAS-lite シャープニング

**Stereo TAAU**:
- 両眼用に独立した TaauPass インスタンス
- 左右のジッタを非相関化 (片眼で frame、もう片眼で frame+8 を使う)
- 16 エントリの Halton テーブルの半分オフセット
- 視差融合時のクロスアイドシマー防止

**GPU-driven rendering**:
- `DrawIndirectCommand` / `DrawIndexedIndirectCommand` 構造体
- `ObjectData` に model matrix + AABB を格納
- GPU 上でカリング → 可視オブジェクトのみ描画

**設計意図**: AAA 商用エンジン相当のレンダリングパイプラインを Rust + wgpu でフルスクラッチ実装。AI アップスケーリングが業界標準化する中で、決定論的 (deterministic) TAAU を残す方針。

---

### 3.7 ce_compute — GPU コンピュート

**役割**: wgpu compute shader 実行基盤。CPU バックエンドも持つ。

**主要モジュール**:
- `backend`: バックエンド抽象化
- `cpu_backend`: CPU 実装
- `wgpu_compute`: wgpu compute 実装

**依存**: wgpu, rayon

**設計意図**: GPU 計算を抽象化し、CPU フォールバックを持つ。物理シミュやワールド生成の汎用計算層。

---

### 3.8 ce_xr — VR/AR (OpenXR)

**役割**: OpenXR 0.19 ベースの VR/AR サポート。商用 VR エンジン相当の機能セット。

**主要モジュール**:
- `body_tracking`: 全身トラッキング (36 関節、Meta Body Tracking API 互換)
- `eye_tracking`: アイトラッキング (gaze direction, pupil dilation, fixation point)
- `face_tracking`: 表情トラッキング (63 blend shapes、ARKit 互換)
- `input`: ハンドスケルトン、ヘッドポーズ
- `session`: OpenXR セッション管理
- `swapchain`: VR スワップチェーン
- `voice`: 音声入力 (VAD、wake word、speech recognition)

**face tracking 詳細**:
- 63 ブレンドシェイプ
- ARKit 標準と Meta Quest Pro/3 (XR_FB_face_tracking2) と HTC Vive (XR_HTC_facial_tracking) の上位互換
- 目、顎、口、頬、舌、眉、鼻のすべて

**body tracking 詳細**:
- 36 関節
- Meta Body Tracking API 互換
- 体幹 (Root, Hips, Spine x3, Chest, Neck, Head) + 両腕 + 両脚
- 手の指関節は `HandSkeleton` で別管理

**eye tracking 詳細**:
- 視線方向 (head-local space)
- 注視点 (両眼の収束点、world space)
- 瞳孔拡張 (0.0 - 1.0、NPC 反応に利用可能)

**voice 詳細**:
- BCP47 言語コード (デフォルト ja-JP)
- 16kHz サンプリング
- ウェイクワード検出
- VAD 閾値

**設計意図**: 商用 VR ヘッドセット (Meta Quest Pro/3、HTC Vive、Pico) で要求される現代的なトラッキング機能をすべて API 化する。OpenXR ランタイムがない環境では graceful degradation し、デスクトップモードで動作する。

---

### 3.9 ce_physics — 物理シミュレーション

**役割**: 剛体物理、衝突判定、空間グリッド、GPU 物理。

**主要モジュール**:
- `collider`: コライダー (形状)
- `collision`: 衝突イベント、接触点
- `gpu_physics`: GPU 上の物理シミュ (GpuBody, GpuPhysics)
- `rigid_body`: 剛体型 (BodyType, PhysicsMaterial, RigidBody, Velocity)
- `spatial`: 空間ハッシュグリッド

**統合方法**:
```rust
PhysicsPlugin {
    gravity: Vec3::new(0.0, -9.81, 0.0),
    fixed_timestep: 1.0 / 60.0,
}
```
- semi-implicit Euler 積分
- FixedUpdate ステージで実行

**設計意図**: CPU パスと GPU パスの両方を持つハイブリッド物理層。rayon による並列化、bytemuck による GPU 転送最適化。

---

### 3.10 ce_chemistry — 化学シミュレーション

**役割**: 元素、原子、結合、分子、化学反応のフルスタック。

**主要型**:

#### ElementProperties (元素プロパティ)
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

#### PeriodicTable
118 元素全てを内包。`ChemistryPlugin` がロード時に Resource として挿入する。

#### Atom (ECS コンポーネント)
```rust
pub struct Atom {
    pub element: ElementId,
    pub charge: i8,
    pub mass_override: Option<f64>,  // 同位体対応
}
```

#### BondType (結合種類、7 種)
- Single, Double, Triple
- Aromatic
- Ionic
- Hydrogen
- VanDerWaals

#### Bond
```rust
pub struct Bond {
    pub atom_a: Entity,
    pub atom_b: Entity,
    pub bond_type: BondType,
    pub bond_order: f64,
    pub equilibrium_length: f64,  // Angstroms
}
```

#### ReactionRule
```rust
pub struct ReactionRule {
    pub name: String,
    pub reactants: Vec<String>,
    pub products: Vec<String>,
    pub activation_energy: f64,    // kJ/mol
    pub enthalpy_change: f64,      // kJ/mol (negative = exothermic)
    pub rate_constant: f64,
}
```

**設計意図**: 化学を「ゲームエンジンに後付けする機能」ではなく、ECS の第一級市民として扱う。原子も結合も Entity として扱え、反応規則も Resource として登録される。これにより、ワールド内の物質を化学的に正しくシミュレートでき、procedural generation の素材層 (L-system + 分子) と直結する。

**業界における位置**:
- Unity / Unreal: 化学シミュレーション層なし
- Bevy: 化学プラグインなし
- 商用 CAD/シミュレータ (Avogadro, GROMACS): 化学はあるがゲームエンジンではない

chemengine はこの中間を埋める存在になっている。

---

### 3.11 ce_ai — NPC 意識カーネル (SMA28 Shion)

**役割**: 仏教心理学に基づいた NPC 意識実装。

**主要モジュール**:

#### consciousness.rs — 三層意識処理
```rust
pub struct Consciousness {
    pub vow_weights: [f64; 4],       // [blessing, perception, justice, continuity]
    pub trait_sediment: [f64; 4],    // 経験からの形質沈殿
    pub vedana: f64,                 // 受 (raw sensation)
    pub felt: f64,                   // 防御で増幅された感覚
    pub filtered: f64,               // 気づきで減衰された信号
    pub defense_strain: f64,         // 防御張力 (0 = 開放、1 = 最大防御)
    pub merit: f64,                  // 功徳
    pub awareness: f64,              // 気づきレベル
    pub prediction_error: f64,       // 予測誤差 EMA
    pub step_count: u64,
}
```

**処理フロー**:
1. **Vedana** (受) — 生の感覚
2. **Felt** — 防御張力で増幅
3. **Filtered** — 気づきで減衰

予測誤差が防御張力の適応と形質沈殿の蓄積を駆動する。これは FEP (Free Energy Principle) の predictive coding と仏教の三層識別の構造同型を実装に落としたもの。

`shion_archetype()` ファクトリで Shion 性格を生成 (blessing と continuity の vow が高い、防御張力やや高め、気づき強化)。

#### three_poisons.rs — 三毒
```rust
pub enum PoisonType {
    Lobha,  // 貪 (greed/attachment)
    Dosa,   // 瞋 (hatred/aversion)
    Moha,   // 痴 (delusion/ignorance)
}

pub struct ThreePoisons {
    pub lobha: f64,
    pub dosa: f64,
    pub moha: f64,
}
```

`villain(dominant)` で支配的な毒を持つ NPC を生成。Default は residual amounts (徳の高い存在も種を持つ)。

#### four_immeasurables.rs — 四無量心 (報酬関数)
```rust
pub struct FourImmeasurables {
    pub maitri: f64,   // 慈 (loving-kindness)
    pub karuna: f64,   // 悲 (compassion)
    pub mudita: f64,   // 喜 (sympathetic joy)
    pub upekkha: f64,  // 捨 (equanimity)
}
```

**報酬計算式**:
```
reward = maitri * happiness_delta
       - karuna * suffering_delta
       + mudita * max(happiness_delta, 0)
       + upekkha * 0.1
```

これは強化学習の報酬関数として、四無量心を直接実装したもの。NPC の意思決定が「他者の幸福を増やし、苦を減らす」方向に駆動される。

`bodhisattva()` で全て最大の理想状態を生成。

#### emotion.rs — Plutchik の感情ホイール
```rust
pub struct EmotionalState {
    pub joy, sadness, anger, fear,
    pub surprise, disgust, trust, anticipation: f32,
}
```

連続値で8基本感情を表現。`blend_toward(target, rate)` で滑らかな遷移。

#### contact_tier.rs — 知識の出自追跡
```rust
pub enum ContactTier {
    FirstHand,  // 直接経験 (最も信頼可)
    Derived,    // 推論から導出
    Hearsay,    // 伝聞 (最も低信頼)
}

pub struct Knowledge {
    pub content: String,
    pub tier: ContactTier,
    pub confidence: f64,
    pub freshness: f64,
    pub source: Option<String>,
}
```

`freshness` は時間で指数減衰する (半減期約 208 秒)。

**設計意図**: ゲーム業界の NPC AI は behavior tree か state machine か LLM ベースに収束しているが、chemengine の ce_ai は仏教心理学を一次設計原理に置く。これは:
- 三層意識処理 (Vedana → Felt → Filtered) が FEP の predictive coding と構造同型
- 四無量心が強化学習の報酬関数として実装可能
- ContactTier が認識論的源泉を追跡する装置になる

業界に類例なし。

---

### 3.12 ce_worldgen — 手続き世界生成

**役割**: 地形、バイオーム、ダンジョン、ノイズ。

**主要モジュール**:
- `biome`: BiomeMap (バイオーム分布)
- `dungeon`: Dungeon, Room (ダンジョン生成)
- `noise`: Noise2D (Perlin/Simplex 系ノイズ)
- `terrain`: TerrainChunk, TerrainConfig (チャンク化地形)

```rust
WorldGenPlugin {
    seed: 42,
    chunk_size: 64,
}
```

**設計意図**: 完全 procedural、テクスチャ画像なし。シェーダで色と光を定義し、L-system と ce_chemistry の分子層が素材を生む方針 (procedural-only direction)。

---

### 3.13 ce_scene — シーン管理

**役割**: Transform, Hierarchy, Camera, Light などのシーングラフ要素。

**設計意図**: ECS の上にシーングラフ的な親子関係を構築する層。

---

### 3.14 ce_interaction — XR + AI 統合層

**役割**: ce_xr の入力 (face/body/eye/voice) と ce_ai の意識カーネルを接続する最上層。

**依存**: ce_ai, ce_xr, ce_app, ce_ecs, ce_core, ce_math

**設計意図**: 「身体性と意識の接続点」。XR で取得した face blend shapes が NPC の Consciousness の vedana 入力に直結する、というような統合がこの層で実装される。

---

## 4. 特徴と差別化点

### 4.1 NPC AI が仏教心理学ベース

業界の NPC AI は:
- Behavior Tree (Unity, Unreal の標準)
- Finite State Machine (古典)
- GOAP / Utility AI (中級)
- LLM-driven (最新)

chemengine の ce_ai は上記のいずれでもない。Vedana / Felt / Filtered の三層処理、三毒、四無量心報酬、ContactTier の知識出自追跡。これは認知アーキテクチャ研究 (Soar, ACT-R) の血筋に近いが、仏教心理学を一次設計原理にする例は他にない。

### 4.2 化学シミュが ECS の第一級市民

118 元素、7 結合タイプ、化学反応規則 (活性化エネルギー / エンタルピー / 速度定数) が ECS Entity / Resource として扱える。これは:
- Unity/Unreal: 化学なし
- Bevy: 化学プラグインなし
- Avogadro/GROMACS: 化学はあるがゲームエンジンではない

chemengine はこの中間を埋める。

### 4.3 AI フリーの決定論的 TAAU

業界が DLSS / FSR / XeSS の AI 推論アップスケーリングに収束する中、chemengine の TAAU は AI 推論を使わない決定論的実装 (Halton ジッタ + YCoCg 近傍クランピング + CAS-lite シャープニング)。

これは:
- 再現性が必要な研究用途
- AI 推論を載せたくない VR (レイテンシ予算)
- 完全決定論シミュレーション

で価値が出る選択。

### 4.4 VR フル機能セット

Quest Pro/3 や HTC Vive の最新機能 (face tracking 63 shapes、body tracking 36 joints、eye tracking with pupil dilation、voice with VAD) を最初から API として備える。OpenXR ランタイムがない環境では graceful degradation してデスクトップモードで動作する。

### 4.5 procedural-only 方針

テクスチャ画像を使わない。シェーダで色と光を定義し、L-system と分子層で素材を生成する。これによりアセットなしで動くゲームエンジンとして完結する。

### 4.6 完全 Rust + wgpu スタック

- Rust edition 2021
- wgpu 24 (最新)
- OpenXR 0.19
- glam 0.29 (math)
- winit 0.30 (windowing)

すべてのコードが unsafe を最小化した安全なRust で書かれている。

---

## 5. 例 (examples/)

6 つの example が同梱されている:

| example | 内容 |
|---------|------|
| hello_triangle | 三角形描画 (最小サンプル) |
| gpu_particles | GPU パーティクルシステム |
| world_demo | 手続き世界生成のデモ |
| benchmark | 性能測定 |
| upscale_demo | TAAU アップスケールデモ |
| vr_stereo_demo | VR ステレオレンダリングデモ |

---

## 6. 全体評価

### 強み
- 14 crates が層構造で整理されている
- ECS / レンダリング / 物理 / 化学 / VR / NPC AI を1スタックに統合
- 業界に類例のない NPC AI 設計 (仏教心理学ベース)
- 業界に類例のない化学シミュ統合
- AAA 商用エンジン相当のレンダリングパイプライン

### 用途想定
- 化学が世界の一部であるゲーム (錬金術、創薬、料理 etc.)
- VR で NPC と対話する作品 (Quest Pro の表情入力 → NPC の vedana)
- 完全 procedural な世界 (アセットレス)
- 研究用途 (決定論的 TAAU、ECS ベンチマーク)

### 制約
- 商用エンジンと比べて成熟度は低い (例えばエディタなし)
- ドキュメンテーションは Rust doc コメントのみ
- アセットパイプラインはなし (procedural-only 方針のため)

---

## 7. リポジトリ統計まとめ

```
Total crates:        14
Total examples:      6
Total .rs files:     83 (in crates/)
Total lines:         14,715
Largest crate:       ce_render (2,384 lines)
Smallest crate:      ce_compute (186 lines)
Most novel crates:   ce_ai (980 lines), ce_chemistry (1,895 lines)
Build profile:       opt-level = 3, lto = "thin"
```

---

**Document end.**
