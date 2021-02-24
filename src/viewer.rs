extern crate piston_window;

use piston_window::*;

/// Viewer is used for rendering the state of an environment
pub struct Viewer {
    pub(crate) window_width: u32,
    pub(crate) window_height: u32,
    pub(crate) window: PistonWindow,
    pub(crate) glyphs: Glyphs,
}

impl Viewer {
    /// Create a new Viewer with given window width and height
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let mut window: PistonWindow = WindowSettings::new("Gym-rs", (window_width, window_height))
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

        let font = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("font")
            .unwrap();
        let glyphs = window.load_font(font.join("anon.ttf")).unwrap();

        Self {
            window_width,
            window_height,
            window,
            glyphs,
        }
    }
}

impl Default for Viewer {
    fn default() -> Self {
        Self::new(600, 400)
    }
}
