use haversine_compute::Point;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::Hasher;
use std::io::Write;

#[derive(Clone, Serialize, Default)]
pub struct JsonResult<'a> {
    pub pairs: &'a [Point],
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
        results: &mut impl Write,
    ) -> Result<f64, BoxDynError>;
}
