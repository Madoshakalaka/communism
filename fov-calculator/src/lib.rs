use derive_more::{Display, From};
use std::f64;
use std::f64::consts::PI;
use std::ops::Div;

#[derive(From)]
pub struct Centimeter(f64);

impl Div for Centimeter {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.0.div(rhs.0)
    }
}

impl Centimeter {
    fn scaled(&self, v: f64) -> Self {
        Self(self.0 * v)
    }
}

#[derive(Display)]
pub struct Degree(f64);

impl Degree {
    fn from_radian(r: f64) -> Self {
        Self(180.0 * r / PI)
    }
}

pub fn calculate_fov<W: Into<Centimeter>, H: Into<Centimeter>, D: Into<Centimeter>>(
    window_width: W,
    window_height: H,
    distance_from_screen: D,
) -> Degree {
    let window_width: Centimeter = window_width.into();
    let window_height: Centimeter = window_height.into();
    let distance_from_screen = distance_from_screen.into();
    let rad = 2.0
        * f64::atan(window_width.scaled(0.5) / distance_from_screen)
        * (window_width / window_height);
    Degree::from_radian(rad)
}
