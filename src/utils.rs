use num::Float;

/// scales a value from one range to another range
/// generic over all float type such as f32 or f64
pub fn scale<T: Float>(from_min: T, from_max: T, to_min: T, to_max: T, value: T) -> T {
    to_min + ((value - from_min) * to_max - to_min) / (from_max - from_min)
}
