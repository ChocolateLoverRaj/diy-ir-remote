use std::time::Duration;

pub trait IsAround {
    fn is_around(&self, other: Self, error: f64) -> bool;
}

impl IsAround for Duration {
    fn is_around(&self, other: Self, error: f64) -> bool {
        let lower_bound = other.mul_f64(1.0 - error);
        let upper_bound = other.mul_f64(1.0 + error);
        self >= &lower_bound && self <= &upper_bound
    }
}
