use crate::types::{BoxDynError, HaversinePointGenerator, JsonResult};
use haversine_compute::{compute_haversine, Point};
use rand::distributions::{Distribution, Uniform};
use std::io::Write;

pub struct UniformHaversinePointsGenerator;

impl HaversinePointGenerator for UniformHaversinePointsGenerator {
    fn generate(
        &self,
        seed: String,
        count: usize,
        output: &mut impl Write,
        results: &mut impl Write,
    ) -> Result<f64, BoxDynError> {
        let mut rng = self.rng_from_seed(seed);

        let latitude_distribution = Uniform::new_inclusive(-90.0, 90.0);
        let longitude_distribution = Uniform::new_inclusive(-180.0, 180.0);

        let mut container = Vec::<Point>::with_capacity(count);
        let mut computed_distances = Vec::<f64>::with_capacity(count);

        for _ in 0..count {
            let (latitude1, longitude1) = (
                latitude_distribution.sample(&mut rng),
                longitude_distribution.sample(&mut rng),
            );

            let (latitude2, longitude2) = (
                latitude_distribution.sample(&mut rng),
                longitude_distribution.sample(&mut rng),
            );

            let point = Point {
                x0: longitude1,
                y0: latitude1,
                x1: longitude2,
                y1: latitude2,
            };

            container.push(point);
            computed_distances.push(compute_haversine(point, 6372.8));
        }

        serde_json::to_writer(
            output,
            &JsonResult {
                pairs: container.as_slice(),
            },
        )?;

        let mut sum = 0.;

        for value in computed_distances {
            results.write_all(format!("{value}\n").as_bytes())?;
            sum += value;
        }

        Ok(sum / count as f64)
    }
}
