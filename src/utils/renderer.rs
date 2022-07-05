struct Renderer<'a> {
    no_returns_render: &'a [RenderMode],
    single_render: &'a [RenderMode],
    mode: RenderMode,
    render: RenderFn,
    render_list: Vec<Render>,
}

type RenderFn = fn(RenderMode) -> Render;

impl<'a> Renderer<'a> {
    pub fn new(
        mode: RenderMode,
        render: RenderFn,
        no_returns_render: Option<&'a [RenderMode]>,
        single_render: Option<&'a [RenderMode]>,
    ) -> Self {
        Self {
            no_returns_render: no_returns_render.unwrap_or(RenderMode::NO_RETURNS_RENDER),
            single_render: single_render.unwrap_or(RenderMode::SINGLE_RENDER),
            mode,
            render,
            render_list: Vec::new(),
        }
    }
    pub fn render_step(&mut self) {
        if self.mode != RenderMode::None && !self.single_render.contains(&self.mode) {
            let render_frame = (self.render)(self.mode);
            if !self.no_returns_render.contains(&self.mode) {
                self.render_list.push(render_frame)
            }
        }
    }

    pub fn get_render(&mut self) -> Option<Vec<Render>> {
        if self.single_render.contains(&self.mode) {
            Some(vec![(self.render)(self.mode)])
        } else if self.mode != RenderMode::None && !self.no_returns_render.contains(&self.mode) {
            let renders = self.render_list;
            self.render_list = Vec::new();
            Some(renders)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.render_list = Vec::new();
    }
}

pub type RenderFrame = [[[usize; 3]; 255]; 255];

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum RenderMode {
    Human,
    SingleRgbArray,
    RgbArray,
    Ansi,
    None,
}

impl RenderMode {
    const NO_RETURNS_RENDER: &'static [RenderMode] = &[RenderMode::Human];
    const SINGLE_RENDER: &'static [RenderMode] = &[RenderMode::SingleRgbArray];
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Render {
    Human,
    SingleRgbArray(RenderFrame),
    RgbArray(Vec<RenderFrame>),
    Ansi(Vec<String>),
    None,
}
