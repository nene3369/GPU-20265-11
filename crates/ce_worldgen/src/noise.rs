//! Deterministic 2D noise generator (value noise with octaves).
//!
//! Uses a seeded LCG-based hash for pure determinism — no external crate needed.
//! This gives reproducible worlds: same seed → same terrain, always.

/// Seeded 2D noise generator.
#[derive(Debug, Clone)]
pub struct Noise2D {
    seed: u64,
    octaves: u32,
    lacunarity: f64,
    persistence: f64,
}

impl Noise2D {
    /// Create a noise generator with default octave settings.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            octaves: 6,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }

    /// Create with custom octave parameters.
    pub fn with_octaves(seed: u64, octaves: u32, lacunarity: f64, persistence: f64) -> Self {
        Self {
            seed,
            octaves,
            lacunarity,
            persistence,
        }
    }

    /// Sample noise at (x, y). Returns value in [0.0, 1.0].
    pub fn sample(&self, x: f64, y: f64) -> f64 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_amplitude = 0.0;

        for octave in 0..self.octaves {
            let sx = x * frequency;
            let sy = y * frequency;
            let value = self.value_noise(sx, sy, octave);
            total += value * amplitude;
            max_amplitude += amplitude;
            frequency *= self.lacunarity;
            amplitude *= self.persistence;
        }

        // Normalize to [0, 1]
        if max_amplitude > 0.0 {
            total / max_amplitude
        } else {
            0.5
        }
    }

    /// Single-octave value noise via grid hashing + bilinear interpolation.
    fn value_noise(&self, x: f64, y: f64, octave: u32) -> f64 {
        let xi = x.floor() as i64;
        let yi = y.floor() as i64;
        let xf = x - x.floor();
        let yf = y - y.floor();

        // Smoothstep for interpolation
        let u = xf * xf * (3.0 - 2.0 * xf);
        let v = yf * yf * (3.0 - 2.0 * yf);

        let v00 = self.hash_to_f64(xi, yi, octave);
        let v10 = self.hash_to_f64(xi + 1, yi, octave);
        let v01 = self.hash_to_f64(xi, yi + 1, octave);
        let v11 = self.hash_to_f64(xi + 1, yi + 1, octave);

        // Bilinear interpolation
        let a = v00 + (v10 - v00) * u;
        let b = v01 + (v11 - v01) * u;
        a + (b - a) * v
    }

    /// Deterministic hash of grid coordinate to [0, 1].
    fn hash_to_f64(&self, x: i64, y: i64, octave: u32) -> f64 {
        let mut h = self.seed;
        h = h
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(x as u64);
        h = h
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(y as u64);
        h = h
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(octave as u64);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        (h & 0xFFFFFFFF) as f64 / 0xFFFFFFFF_u64 as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_deterministic() {
        let n = Noise2D::new(42);
        let a = n.sample(1.5, 2.7);
        let b = n.sample(1.5, 2.7);
        assert_eq!(a, b);
    }

    #[test]
    fn noise_in_range() {
        let n = Noise2D::new(123);
        for i in 0..100 {
            let v = n.sample(i as f64 * 0.1, i as f64 * 0.3);
            assert!(v >= 0.0 && v <= 1.0, "noise out of range: {v}");
        }
    }

    #[test]
    fn different_seeds_different_output() {
        let n1 = Noise2D::new(1);
        let n2 = Noise2D::new(2);
        let v1 = n1.sample(5.0, 5.0);
        let v2 = n2.sample(5.0, 5.0);
        assert!((v1 - v2).abs() > 1e-6, "different seeds should differ");
    }

    #[test]
    fn noise_is_smooth() {
        let n = Noise2D::new(42);
        let a = n.sample(1.0, 1.0);
        let b = n.sample(1.001, 1.0);
        assert!((a - b).abs() < 0.1, "noise should be smooth: {a} vs {b}");
    }
}
