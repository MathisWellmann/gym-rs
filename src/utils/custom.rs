use std::marker::PhantomData;

use derivative::Derivative;
use derive_new::new;
use ordered_float::OrderedFloat;
use rand_pcg::Pcg64;
use sdl2::{
    gfx::framerate::FPSManager, pixels::PixelFormatEnum, render::WindowCanvas, EventPump,
    EventSubsystem,
};
use serde::Serialize;

use super::renderer::{RenderColor, RenderFrame, RenderMode};

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

pub struct ScreenGui {
    pub canvas: WindowCanvas,
    pub fps_manager: FPSManager,
    pub event_pump: EventPump,
    pub event_subsystem: EventSubsystem,
}

#[derive(Serialize, Derivative)]
#[derivative(Debug)]
pub struct Screen {
    pub height: u32,
    pub width: u32,
    pub title: &'static str,
    #[serde(skip_serializing)]
    #[derivative(Debug = "ignore")]
    pub gui: Option<ScreenGui>,
}

impl Screen {
    pub fn new(height: u32, width: u32, title: &'static str) -> Self {
        Self {
            height,
            width,
            title,
            gui: None,
        }
    }
}

impl Clone for Screen {
    fn clone(&self) -> Self {
        Self {
            height: self.height.clone(),
            width: self.width.clone(),
            title: self.title.clone(),
            gui: None,
        }
    }
}

pub trait DefaultSeed {
    fn default(rng: &mut Pcg64) -> Self;
}

impl Screen {
    pub fn is_open(&self) -> bool {
        self.gui.is_none()
    }
}

impl Screen {
    pub fn default(
        height: u32,
        width: u32,
        title: &'static str,
        render_fps: u32,
        mode: RenderMode,
    ) -> Self {
        let context = sdl2::init().unwrap();
        let video_subsystem = context.video().unwrap();
        let mut window_builder = video_subsystem.window(&title, width, height);

        window_builder.position_centered();

        if mode != RenderMode::Human {
            window_builder.hidden();
        }

        let window = window_builder.build().unwrap();
        let canvas = window.into_canvas().accelerated().build().unwrap();
        let event_pump = context.event_pump().expect("Could not recieve event pump.");
        let mut fps_manager = FPSManager::new();
        let event_subsystem = context
            .event()
            .expect("Event subsystem was not initialized.");
        fps_manager
            .set_framerate(render_fps)
            .expect("Framerate was unable to be set.");

        Screen {
            gui: Some(ScreenGui {
                canvas,
                event_pump,
                event_subsystem,
                fps_manager,
            }),
            height,
            width,
            title,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Ord, PartialOrd, Copy, new)]
pub struct Metadata<T> {
    pub render_modes: &'static [RenderMode],
    pub render_fps: u32,
    marker: PhantomData<T>,
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
