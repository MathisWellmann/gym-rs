use num::Float;

/// scales a value from one range to another range
/// generic over all float type such as f32 or f64
pub fn scale<T: Float>(from_min: T, from_max: T, to_min: T, to_max: T, value: T) -> T {
    to_min + ((value - from_min) * (to_max - to_min)) / (from_max - from_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale() {
        let s = scale(0.0, 1.0, 0.0, 100.0, 0.5);
        assert_eq!(s, 50.0,);

        let s = scale(-1.0, 1.0, 0.0, 1.0, 0.0);
        assert_eq!(s, 0.5);
    }
}