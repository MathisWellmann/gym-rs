use crate::utils::renderer::{Render, RenderMode};

pub trait GymEnv {
    type Action;

    fn step(&mut self, action: Self::Action) -> (Vec<f64>, f64, bool, Option<String>);

    fn reset(&mut self) -> Vec<f64>;

    fn render(&self, mode: RenderMode) -> Render;

    fn seed(&mut self, seed: Option<u64>) -> Vec<u64>;
}
