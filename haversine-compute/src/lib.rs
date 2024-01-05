use instrument_macros::instrument;
use serde::Serialize;

#[derive(Copy, Clone, Serialize, Default)]
pub struct Point {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[cfg_attr(feature = "profile", instrument)]
#[must_use]
pub fn compute_haversine(Point { x0, y0, x1, y1 }: Point, earth_radius: f64) -> f64 {
    let delta_latitude = y1 - y0;
    let delta_longitude = x1 - x0;

    let haversine_theta = (delta_latitude / 2.).to_radians().sin().powi(2)
        + (y1.to_radians().cos()
            * y0.to_radians().cos()
            * (delta_longitude / 2.).to_radians().sin().powi(2));

    let unit_distance = 2. * haversine_theta.sqrt().asin();

    earth_radius * unit_distance
}
