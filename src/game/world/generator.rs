use noise::{NoiseFn, OpenSimplex, Perlin, Seedable};

pub struct TerrainGenerator {
    noise: OpenSimplex
}

impl TerrainGenerator {
    pub fn get_height(&self, x: i32, z: i32) -> f32 {
        let noise_x = (x as f64) / 20.;
        let noise_z = (z as f64) / 20.;
        let noise_height = self.noise.get([noise_x, noise_z]) as f32;
        return (noise_height + 1.) * 20.;
    }
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            noise: OpenSimplex::new().set_seed(12345)
        }
    }
}