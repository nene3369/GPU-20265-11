//! ce_worldgen — Procedural world generation for ChemEngine.
//!
//! Generates terrain, biomes, dungeons, and settlements from a seed.

pub mod biome;
pub mod dungeon;
pub mod noise;
pub mod terrain;

pub use biome::{Biome, BiomeMap};
pub use dungeon::{Dungeon, Room};
pub use noise::Noise2D;
pub use terrain::{TerrainChunk, TerrainConfig};

use ce_app::{App, Plugin};

/// World generation plugin.
pub struct WorldGenPlugin {
    pub seed: u64,
    pub chunk_size: u32,
}

impl Default for WorldGenPlugin {
    fn default() -> Self {
        Self {
            seed: 42,
            chunk_size: 64,
        }
    }
}

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldGenConfig {
            seed: self.seed,
            chunk_size: self.chunk_size,
        });
        log::info!(
            "WorldGenPlugin loaded: seed={}, chunk={}",
            self.seed,
            self.chunk_size
        );
    }
}

/// Global world generation configuration resource.
#[derive(Debug, Clone)]
pub struct WorldGenConfig {
    pub seed: u64,
    pub chunk_size: u32,
}
