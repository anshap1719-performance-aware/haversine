use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::Hasher;
use std::io::Write;

#[derive(Copy, Clone, Serialize, Default)]
pub struct Point {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[derive(Clone, Serialize, Default)]
pub struct JsonResult {
    pub pairs: Vec<Point>,
}

pub type BoxDynError = Box<dyn Error>;

pub trait HaversinePointGenerator
where
    Self: Sized,
{
    fn rng_from_seed(&self, seed: String) -> StdRng {
        let mut hasher = DefaultHasher::new();
        hasher.write(seed.as_bytes());

        let hash = hasher.finish();

        StdRng::seed_from_u64(hash)
    }

    fn generate(
        &self,
        seed: String,
        count: usize,
        output: &mut impl Write,
    ) -> Result<f64, BoxDynError>;
}
