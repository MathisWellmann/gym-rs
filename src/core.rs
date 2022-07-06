use rand_pcg::Pcg64;

use crate::utils::renderer::{Render, RenderMode};

/// TODO
pub trait Env {
    /// TODO
    type Action;
    type Observation;
    type Info;

    /// TODO
    fn step(&mut self, action: Self::Action) -> ActionReward<Self::Observation, Self::Info>;

    /// TODO
    fn reset(&mut self) -> Vec<f64>;

    /// TODO
    fn render(&mut self, mode: RenderMode) -> Render;

    /// TODO
    fn seed(&mut self, seed: Option<u64>) -> u64;
}

/// TODO
#[derive(Clone, Debug, Copy)]
pub struct ActionReward<T, E> {
    // TODO
    pub observation: T,
    // TODO
    pub reward: f64,
    // TODO
    pub done: bool,
    // TODO
    pub info: Option<E>,
}

trait Seedable {
    fn rand_random(&self) -> &Pcg64;
    fn set_rand_random(&mut self, generator: Pcg64);
}

trait Feedback {}
