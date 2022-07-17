use std::marker::PhantomData;

use derivative::Derivative;
use derive_new::new;
use ordered_float::OrderedFloat;
use rand::distributions::uniform::SampleUniform;
use rand_pcg::Pcg64;
use sdl2::{
    event::Event,
    gfx::framerate::FPSManager,
    pixels::PixelFormatEnum,
    rect::{Point, Rect},
    render::WindowCanvas,
    EventPump, EventSubsystem,
};
use serde::Serialize;

use crate::spaces::BoxR;

use super::renderer::{RenderColor, RenderFrame, RenderMode, Renders};

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

#[derive(new)]
pub struct ScreenGuiTransformations {
    src: Option<Rect>,
    dst: Option<Rect>,
    angle: f64,
    center: Option<Point>,
    flip_horizontal: bool,
    flip_vertical: bool,
}

impl ScreenGuiTransformations {
    pub fn with_flip_vertical(self, flip_vertical: bool) -> Self {
        Self {
            flip_vertical,
            ..self
        }
    }
}

impl Default for ScreenGuiTransformations {
    fn default() -> Self {
        Self::new(None, None, 0., None, false, true)
    }
}

impl ScreenGuiTransformations {
    pub fn center(&self) -> Option<Point> {
        self.center
    }
}

#[derive(Serialize, Derivative, new)]
#[derivative(Debug)]
pub struct Screen {
    pub height: u32,
    pub width: u32,
    pub title: &'static str,
    pub render_fps: u32,
    pub mode: RenderMode,
    #[serde(skip_serializing)]
    #[derivative(Debug = "ignore")]
    #[new(default)]
    pub gui: Option<ScreenGui>,
}

impl Clone for Screen {
    fn clone(&self) -> Self {
        Self {
            height: self.height.clone(),
            width: self.width.clone(),
            title: self.title.clone(),
            render_fps: self.render_fps.clone(),
            mode: self.mode.clone(),
            gui: None,
        }
    }
}

pub trait Sample: SampleUniform {
    fn sample_between(rng: &mut Pcg64, bounds: Option<BoxR<Self>>) -> Self;
}

impl Screen {
    pub fn is_open(&self) -> bool {
        self.gui.is_some()
    }

    pub fn render(&mut self, mode: RenderMode) -> Renders {
        match self.gui.as_mut() {
            Some(ScreenGui {
                canvas,
                fps_manager,
                ..
            }) => {
                fps_manager.delay();
                canvas.present();
                if [RenderMode::RgbArray, RenderMode::SingleRgbArray].contains(&mode) {
                    Renders::SingleRgbArray(canvas_to_pixels(canvas, self.width))
                } else {
                    Renders::None
                }
            }
            _ => Renders::None,
        }
    }

    pub fn draw_on_canvas(
        &mut self,
        using_fn: impl FnMut(&mut WindowCanvas) -> (),
        with_transformations: ScreenGuiTransformations,
    ) {
        match self.gui.as_mut() {
            Some(ScreenGui { canvas, .. }) => {
                let texture_creator = canvas.texture_creator();
                let mut texture = texture_creator
                    .create_texture_target(PixelFormatEnum::RGB24, self.width, self.height)
                    .expect("Create texture.");

                canvas
                    .with_texture_canvas(&mut texture, using_fn)
                    .expect("Was unable to render.");

                canvas
                    .copy_ex(
                        &mut texture,
                        with_transformations.src,
                        with_transformations.dst,
                        with_transformations.angle,
                        with_transformations.center,
                        with_transformations.flip_horizontal,
                        with_transformations.flip_vertical,
                    )
                    .expect("Transformations failed to be applied.");
            }
            _ => (),
        }
    }

    pub fn consume_events(&mut self) {
        match self.gui.as_mut() {
            Some(ScreenGui { event_pump, .. }) => {
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => {
                            panic!("ANIMATION WAS FORCED TO EXIT!")
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    pub fn load_gui(&mut self) {
        if self.gui.is_none() {
            let title = self.title;
            let width = self.width;
            let height = self.height;
            let render_fps = self.render_fps;
            let mode = self.mode;

            let gui = {
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

                ScreenGui {
                    canvas,
                    event_pump,
                    event_subsystem,
                    fps_manager,
                }
            };

            self.gui = Some(gui);
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
