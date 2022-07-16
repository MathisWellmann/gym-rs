use ordered_float::OrderedFloat;
use sdl2::{pixels::PixelFormatEnum, render::WindowCanvas};

use super::renderer::{RenderColor, RenderFrame};

pub type O64 = OrderedFloat<f64>;

/// scales a value from one range to another range
/// generic over all float type such as f32 or f64
pub fn scale<T: num::Float>(from_min: T, from_max: T, to_min: T, to_max: T, value: T) -> T {
    to_min + ((value - from_min) * (to_max - to_min)) / (from_max - from_min)
}

/// TODO: Write documentation
pub fn clip<T: PartialEq + PartialOrd>(value: T, left_bound: T, right_bound: T) -> T {
    if left_bound <= value && value <= right_bound {
        value
    } else if value > right_bound {
        right_bound
    } else {
        left_bound
    }
}

pub fn canvas_to_pixels(canvas: &mut WindowCanvas, screen_width: u32) -> RenderFrame {
    let pixels = canvas
        .read_pixels(None, PixelFormatEnum::RGB24)
        .expect("pixels");

    let colours: Vec<RenderColor> = pixels
        .chunks(3)
        .map(|chunk| RenderColor::RGB(chunk[0], chunk[1], chunk[2]))
        .collect();

    let pixels_array: Vec<Vec<RenderColor>> = colours
        .chunks(screen_width as usize)
        .map(|chunk| chunk.into())
        .collect();

    RenderFrame::new(pixels_array)
}

pub struct Screen {
    pub canvas: WindowCanvas,
    pub fps_manager: FPSManager,
    pub event_pump: EventPump,
    pub event_subsystem: EventSubsystem,
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
