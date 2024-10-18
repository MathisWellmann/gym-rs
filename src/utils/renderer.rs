use derive_new::new;
use serde::Serialize;

/// A structure which lazily invokes renders and stores the resulting frames.
#[derive(Debug, Serialize, Clone)]
pub struct Renderer {
    /// A list of render modes which should not produce any frames.
    no_returns_render: Vec<RenderMode>,
    /// A list of render modes which produce exactly one frame when it's associated render closure
    /// is called.
    single_render: Vec<RenderMode>,
    /// The render mode associated with the environment holding this render-responsible object.
    mode: RenderMode,
    /// The renders produced in cronological order.
    render_list: Vec<RenderFrame>,
}

/// Describes a lifetime associated closure which takes in a render-mode,
/// extracts the required details from the environment's state and produces the frame associated
/// with the render-mode.
type RenderFn<'a> = &'a mut dyn FnMut(RenderMode) -> Renders;

/// Describes the render-specific operations.
impl Renderer {
    /// Constructs an instance of the Renderer object with an empty set of frames.
    pub fn new(
        mode: RenderMode,
        no_returns_render: Option<Vec<RenderMode>>,
        single_render: Option<Vec<RenderMode>>,
    ) -> Self {
        Self {
            no_returns_render: no_returns_render.unwrap_or(RenderMode::NO_RETURNS_RENDER.to_vec()),
            single_render: single_render.unwrap_or(RenderMode::SINGLE_RENDER.to_vec()),
            mode,
            render_list: Vec::new(),
        }
    }

    /// Renders a frame using the given render closure if structure conditions are met.
    pub fn render_step(&mut self, render: RenderFn) {
        if self.mode != RenderMode::None && !self.single_render.contains(&self.mode) {
            let render_return = render(self.mode);
            if !self.no_returns_render.contains(&self.mode) {
                if let Renders::SingleRgbArray(frame) = render_return {
                    self.render_list.push(frame)
                }
            }
        }
    }

    /// Supplies the list of frames collected thus far.
    pub fn get_renders(&mut self, render: RenderFn) -> Renders {
        if self.single_render.contains(&self.mode) {
            render(self.mode)
        } else if self.mode != RenderMode::None && !self.no_returns_render.contains(&self.mode) {
            let renders = self.render_list.clone();
            self.render_list = Vec::new();
            Renders::RgbArray(renders)
        } else {
            Renders::None
        }
    }

    /// Empties the collection of frames collected.
    pub fn reset(&mut self) {
        self.render_list = Vec::new();
    }
}

/// Defines various palettes capabling of describing the colour of a pixel.
#[derive(Debug, new, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderColor {
    /// Holds the red-green-blue values of some pixel.
    RGB(u8, u8, u8),
}

/// A surface which holds pixels describing the contents produced during a render.
#[derive(Debug, new, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize)]
pub struct RenderFrame(pub Vec<Vec<RenderColor>>);

/// A collection of various formats describing the type of content produced during a render.
#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Serialize, Eq, Ord)]
pub enum RenderMode {
    /// Indicates that that renderer should be done through the terminal or an external display.
    Human,
    /// Indicates that the renderer should produce a single frame containing RGB pixels which describe the
    /// current state of the environment.
    SingleRgbArray,
    /// Indicates that the renderer should produce a list containing all the frames collected
    /// starting from the last reset.
    RgbArray,
    /// Indicates that the renderer should produce a text-representation of the contents found in the
    /// environments current state
    Ansi,
    /// Indicates that renderer should be skipped.
    None,
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::None
    }
}

impl RenderMode {
    /// Defines an empty set of render modes.
    ///
    /// This is useful for when you want to create an renderer which shouldn't
    /// support any rendering at all.
    pub const DEFAULT: &'static [RenderMode] = &[];

    const NO_RETURNS_RENDER: &'static [RenderMode] = &[RenderMode::Human];
    const SINGLE_RENDER: &'static [RenderMode] = &[RenderMode::SingleRgbArray];
}

/// Defines a collection of common structures produced during rendering.
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Ord, Eq)]
pub enum Renders {
    /// Defines the structure produced during the use of the SingleRgbArray mode.
    SingleRgbArray(RenderFrame),
    /// Defines the collection of frames produced during the RgbArray mode.
    RgbArray(Vec<RenderFrame>),
    /// Defines the collection of row strings produced to represent the contents
    /// of the environment when using Ansi mode.
    Ansi(Vec<String>),
    /// Indicates that no render has been produced.
    ///
    /// Most commonly used during Human or None mode.
    None,
}
