pub trait Area {
    fn area(&self) -> f64;
}

pub struct Circle {
    pub radius: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

pub struct Rect {
    pub width: f64,
    pub height: f64,
}

impl Area for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

pub fn total_area<T: Area>(shapes: &[T]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

pub fn apply_transform(values: &[f64], f: impl Fn(f64) -> f64) -> Vec<f64> {
    values.iter().map(|v| f(*v)).collect()
}

pub fn scale_areas<T: Area>(shapes: &[T], factor: f64) -> Vec<f64> {
    let areas: Vec<f64> = shapes.iter().map(|s| s.area()).collect();
    apply_transform(&areas, |a| a * factor)
}
