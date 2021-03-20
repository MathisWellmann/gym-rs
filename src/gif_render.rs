use failure::Error;
use plotters::coord::Shift;
use plotters::prelude::*;

/// Viewer is used for rendering the state of an environment
pub struct GifRender<'a> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) drawing_area: DrawingArea<BitMapBackend<'a>, Shift>,
}

impl<'a> GifRender<'a> {
    /// Create a new Viewer with given window width and height
    pub fn new(width: u32, height: u32, filename: &str, frame_time_ms: u32) -> Result<Self, Error> {
        let drawing_area =
            BitMapBackend::gif(&filename, (width, height), frame_time_ms)?.into_drawing_area();

        Ok(Self {
            width,
            height,
            drawing_area,
        })
    }
}
