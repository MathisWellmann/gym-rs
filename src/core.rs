use rand_pcg::Pcg64;

use crate::utils::renderer::{Render, RenderMode};

/// TODO
pub trait Env {
    /// TODO
    type Action;

    /// TODO
    fn step(&mut self, action: Self::Action) -> (Vec<f64>, f64, bool, Option<String>);

    /// TODO
    fn reset(&mut self) -> Vec<f64>;

    /// TODO
    fn render(&mut self, mode: RenderMode) -> Render;

    /// TODO
    fn seed(&mut self, seed: Option<u64>) -> u64;
}

trait Seedable {
    fn rand_random(&self) -> &Pcg64;
    fn set_rand_random(&mut self, generator: Pcg64);
}

trait Feedback {}
