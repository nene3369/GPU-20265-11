//! Terrain generation using layered noise.

use crate::noise::Noise2D;

/// Configuration for terrain generation.
#[derive(Debug, Clone)]
pub struct TerrainConfig {
    pub seed: u64,
    pub chunk_size: u32,
    /// Height scale in world units.
    pub height_scale: f64,
    /// Frequency of the base terrain noise.
    pub frequency: f64,
    /// Sea level as a fraction of height_scale (0.0 - 1.0).
    pub sea_level: f64,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            chunk_size: 64,
            height_scale: 100.0,
            frequency: 0.01,
            sea_level: 0.35,
        }
    }
}

/// A chunk of terrain heightmap data.
#[derive(Debug, Clone)]
pub struct TerrainChunk {
    /// Chunk position in chunk coordinates.
    pub chunk_x: i32,
    pub chunk_z: i32,
    /// Size of one side (chunk is size × size).
    pub size: u32,
    /// Height values in row-major order, normalized [0.0, 1.0].
    pub heights: Vec<f64>,
}

impl TerrainChunk {
    /// Generate a terrain chunk at the given chunk coordinates.
    pub fn generate(config: &TerrainConfig, chunk_x: i32, chunk_z: i32) -> Self {
        let size = config.chunk_size;
        let noise = Noise2D::new(config.seed);
        let mut heights = Vec::with_capacity((size * size) as usize);

        for z in 0..size {
            for x in 0..size {
                let wx = (chunk_x as f64 * size as f64 + x as f64) * config.frequency;
                let wz = (chunk_z as f64 * size as f64 + z as f64) * config.frequency;
                let h = noise.sample(wx, wz);
                heights.push(h);
            }
        }

        Self {
            chunk_x,
            chunk_z,
            size,
            heights,
        }
    }

    /// Get height at local (x, z) within the chunk. Returns None if out of bounds.
    pub fn get_height(&self, x: u32, z: u32) -> Option<f64> {
        if x < self.size && z < self.size {
            Some(self.heights[(z * self.size + x) as usize])
        } else {
            None
        }
    }

    /// Minimum height in the chunk.
    pub fn min_height(&self) -> f64 {
        self.heights.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// Maximum height in the chunk.
    pub fn max_height(&self) -> f64 {
        self.heights
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Whether a given height is below sea level.
    pub fn is_underwater(&self, x: u32, z: u32, sea_level: f64) -> bool {
        self.get_height(x, z).is_some_and(|h| h < sea_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_generation_is_deterministic() {
        let config = TerrainConfig::default();
        let c1 = TerrainChunk::generate(&config, 0, 0);
        let c2 = TerrainChunk::generate(&config, 0, 0);
        assert_eq!(c1.heights, c2.heights);
    }

    #[test]
    fn chunk_has_correct_size() {
        let config = TerrainConfig {
            chunk_size: 32,
            ..Default::default()
        };
        let chunk = TerrainChunk::generate(&config, 0, 0);
        assert_eq!(chunk.heights.len(), 32 * 32);
    }

    #[test]
    fn heights_in_range() {
        let config = TerrainConfig::default();
        let chunk = TerrainChunk::generate(&config, 5, -3);
        for &h in &chunk.heights {
            assert!(h >= 0.0 && h <= 1.0, "height out of range: {h}");
        }
    }

    #[test]
    fn different_chunks_differ() {
        let config = TerrainConfig::default();
        let c1 = TerrainChunk::generate(&config, 0, 0);
        let c2 = TerrainChunk::generate(&config, 10, 10);
        assert_ne!(c1.heights, c2.heights);
    }

    #[test]
    fn get_height_bounds_check() {
        let config = TerrainConfig {
            chunk_size: 8,
            ..Default::default()
        };
        let chunk = TerrainChunk::generate(&config, 0, 0);
        assert!(chunk.get_height(0, 0).is_some());
        assert!(chunk.get_height(7, 7).is_some());
        assert!(chunk.get_height(8, 0).is_none());
    }

    #[test]
    fn min_max_height() {
        let config = TerrainConfig::default();
        let chunk = TerrainChunk::generate(&config, 0, 0);
        assert!(chunk.min_height() <= chunk.max_height());
        assert!(chunk.min_height() >= 0.0);
        assert!(chunk.max_height() <= 1.0);
    }
}
