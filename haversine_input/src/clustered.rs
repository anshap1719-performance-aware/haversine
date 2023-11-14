use crate::types::{BoxDynError, HaversinePointGenerator, JsonResult};
use haversine_compute::{compute_haversine, Point};
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::io::Write;

pub struct ClusteredHaversinePointsGenerator;

impl HaversinePointGenerator for ClusteredHaversinePointsGenerator {
    fn generate(
        &self,
        seed: String,
        count: usize,
        output: &mut impl Write,
        results: &mut impl Write,
    ) -> Result<f64, BoxDynError> {
        let mut rng = self.rng_from_seed(seed);
        let mut cluster_rng = thread_rng();

        let cluster_count = count.next_power_of_two().ilog2();

        println!("{cluster_count}");

        let clusters: Vec<(f64, (f64, f64))> = (0..cluster_count)
            .map(|_| {
                (
                    cluster_rng.gen_range(20.0..=180.0),
                    (
                        cluster_rng.gen_range(-180.0..=180.0),
                        cluster_rng.gen_range(-90.0..=90.0),
                    ),
                )
            })
            .collect();

        let mut container = vec![Point::default(); count];

        let container = container.as_mut_slice();

        for (chunk, (size, (x, y))) in container.chunks_mut(count / clusters.len()).zip(clusters) {
            let latitude_distribution = Uniform::new_inclusive(y - size, y + size);
            let longitude_distribution = Uniform::new_inclusive(x - size, x + size);

            for Point { x0, x1, y0, y1 } in chunk {
                let (latitude1, longitude1) = (
                    latitude_distribution.sample(&mut rng),
                    longitude_distribution.sample(&mut rng),
                );

                let (latitude2, longitude2) = (
                    latitude_distribution.sample(&mut rng),
                    longitude_distribution.sample(&mut rng),
                );

                *x0 = longitude1;
                *y0 = latitude1;
                *x1 = longitude2;
                *y1 = latitude2;
            }
        }

        let computed_distances: Vec<f64> = container
            .iter()
            .map(|point| compute_haversine(*point, 6372.8))
            .collect();

        serde_json::to_writer(output, &JsonResult { pairs: container })?;

        let mut sum = 0.;

        for value in computed_distances {
            results.write_all(format!("{value}\n").as_bytes())?;
            sum += value;
        }

        Ok(sum / count as f64)
    }
}
