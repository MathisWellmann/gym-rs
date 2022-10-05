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

    /// TODO
    pub fn render_step<'b>(&mut self, render: RenderFn<'b>) {
        if self.mode != RenderMode::None && !self.single_render.contains(&self.mode) {
            let render_return = render(self.mode);
            if !self.no_returns_render.contains(&self.mode) {
                match render_return {
                    Renders::SingleRgbArray(frame) => self.render_list.push(frame),
                    _ => (),
                }
            }
        }
    }

    /// TODO
    pub fn get_renders<'b>(&mut self, render: RenderFn<'b>) -> Renders {
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

    /// TODO
    pub fn reset(&mut self) {
        self.render_list = Vec::new();
    }
}

#[derive(Debug, new, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderColor {
    RGB(u8, u8, u8),
}

/// TODO
#[derive(Debug, new, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize)]
pub struct RenderFrame(pub Vec<Vec<RenderColor>>);

/// TODO
#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Serialize, Eq, Ord)]
pub enum RenderMode {
    /// TODO
    Human,
    /// TODO
    SingleRgbArray,
    /// TODO
    RgbArray,
    /// TODO
    Ansi,
    /// TODO
    None,
    /// TODO
    DepthArray,
    /// TODO
    SingleDepthArray,
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::None
    }
}

impl RenderMode {
    const NO_RETURNS_RENDER: &'static [RenderMode] = &[RenderMode::Human];
    const SINGLE_RENDER: &'static [RenderMode] = &[RenderMode::SingleRgbArray];
    pub const DEFAULT: &'static [RenderMode] = &[];
}

/// TODO
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Ord, Eq)]
pub enum Renders {
    /// TODO
    SingleRgbArray(RenderFrame),
    /// TODO
    RgbArray(Vec<RenderFrame>),
    /// TODO
    Ansi(Vec<String>),
    /// TODO
    None,
}
