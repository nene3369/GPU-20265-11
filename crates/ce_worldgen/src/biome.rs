//! Biome classification using Whittaker diagram (temperature × precipitation).

use crate::noise::Noise2D;
use crate::terrain::TerrainChunk;

/// Biome type based on Whittaker classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Ocean,
    Beach,
    Desert,
    Savanna,
    Grassland,
    TemperateForest,
    TropicalRainforest,
    Taiga,
    Tundra,
    SnowyMountain,
    Swamp,
    Volcano,
}

impl Biome {
    /// Classify biome from temperature, precipitation, and height.
    ///
    /// - `temperature`: 0.0 (frozen) to 1.0 (tropical)
    /// - `precipitation`: 0.0 (arid) to 1.0 (very wet)
    /// - `height`: 0.0 (sea floor) to 1.0 (peak)
    /// - `sea_level`: threshold for ocean/beach
    pub fn classify(temperature: f64, precipitation: f64, height: f64, sea_level: f64) -> Self {
        if height < sea_level - 0.05 {
            return Biome::Ocean;
        }
        if height < sea_level + 0.02 {
            return Biome::Beach;
        }
        if height > 0.85 {
            if temperature < 0.3 {
                return Biome::SnowyMountain;
            }
            return Biome::Volcano; // hot + high = volcanic
        }

        match (temperature > 0.5, precipitation > 0.5) {
            (true, true) => {
                if precipitation > 0.75 {
                    Biome::TropicalRainforest
                } else if height < 0.4 {
                    Biome::Swamp
                } else {
                    Biome::TemperateForest
                }
            }
            (true, false) => {
                if precipitation < 0.2 {
                    Biome::Desert
                } else {
                    Biome::Savanna
                }
            }
            (false, true) => {
                if temperature < 0.2 {
                    Biome::Tundra
                } else {
                    Biome::Taiga
                }
            }
            (false, false) => {
                if temperature < 0.2 {
                    Biome::Tundra
                } else {
                    Biome::Grassland
                }
            }
        }
    }
}

/// A biome map for a terrain chunk.
#[derive(Debug, Clone)]
pub struct BiomeMap {
    pub size: u32,
    pub biomes: Vec<Biome>,
}

impl BiomeMap {
    /// Generate biome map for a terrain chunk.
    pub fn generate(chunk: &TerrainChunk, seed: u64, sea_level: f64) -> Self {
        let temp_noise = Noise2D::with_octaves(seed.wrapping_add(1000), 4, 2.0, 0.5);
        let precip_noise = Noise2D::with_octaves(seed.wrapping_add(2000), 4, 2.0, 0.5);
        let size = chunk.size;
        let mut biomes = Vec::with_capacity((size * size) as usize);

        for z in 0..size {
            for x in 0..size {
                let wx = (chunk.chunk_x as f64 * size as f64 + x as f64) * 0.005;
                let wz = (chunk.chunk_z as f64 * size as f64 + z as f64) * 0.005;

                let temperature = temp_noise.sample(wx, wz);
                let precipitation = precip_noise.sample(wx, wz);
                let height = chunk.heights[(z * size + x) as usize];

                biomes.push(Biome::classify(
                    temperature,
                    precipitation,
                    height,
                    sea_level,
                ));
            }
        }

        Self { size, biomes }
    }

    /// Get biome at local (x, z).
    pub fn get(&self, x: u32, z: u32) -> Option<Biome> {
        if x < self.size && z < self.size {
            Some(self.biomes[(z * self.size + x) as usize])
        } else {
            None
        }
    }

    /// Count occurrences of each biome type.
    pub fn biome_counts(&self) -> std::collections::HashMap<Biome, usize> {
        let mut counts = std::collections::HashMap::new();
        for &b in &self.biomes {
            *counts.entry(b).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_ocean() {
        assert_eq!(Biome::classify(0.5, 0.5, 0.2, 0.35), Biome::Ocean);
    }

    #[test]
    fn classify_beach() {
        assert_eq!(Biome::classify(0.5, 0.5, 0.36, 0.35), Biome::Beach);
    }

    #[test]
    fn classify_desert() {
        assert_eq!(Biome::classify(0.8, 0.1, 0.5, 0.35), Biome::Desert);
    }

    #[test]
    fn classify_snowy_mountain() {
        assert_eq!(Biome::classify(0.1, 0.5, 0.9, 0.35), Biome::SnowyMountain);
    }

    #[test]
    fn classify_tropical_rainforest() {
        assert_eq!(
            Biome::classify(0.9, 0.9, 0.5, 0.35),
            Biome::TropicalRainforest
        );
    }

    #[test]
    fn biome_map_generation() {
        let config = crate::terrain::TerrainConfig {
            chunk_size: 16,
            ..Default::default()
        };
        let chunk = TerrainChunk::generate(&config, 0, 0);
        let map = BiomeMap::generate(&chunk, 42, 0.35);
        assert_eq!(map.biomes.len(), 16 * 16);
        assert!(map.biome_counts().len() >= 1);
    }

    #[test]
    fn biome_map_deterministic() {
        let config = crate::terrain::TerrainConfig {
            chunk_size: 16,
            ..Default::default()
        };
        let chunk = TerrainChunk::generate(&config, 0, 0);
        let m1 = BiomeMap::generate(&chunk, 42, 0.35);
        let m2 = BiomeMap::generate(&chunk, 42, 0.35);
        assert_eq!(m1.biomes, m2.biomes);
    }
}
