pub trait GymEnv {
    type Action;

    fn step(&mut self, action: Self::Action) -> (Vec<f64>, f64, bool, Option<String>);

    fn reset(&mut self) -> Vec<f64>;

    fn render(&self, mode: RenderMode) -> Render;

    fn seed(&mut self, seed: Option<u64>) -> Vec<u64>;
}

pub type RenderFrame = [[[usize; 3]; 255]; 255];

pub enum RenderMode {
    Human,
    SingleRgbArray,
    RgbArray,
    Ansi,
    None,
}

pub enum Render {
    Human,
    SingleRgbArray(RenderFrame),
    RgbArray(Vec<RenderFrame>),
    Ansi(Vec<String>),
    None,
}
