use rand_pcg::Pcg64;
use serde::Serialize;

use crate::utils::renderer::{Render, RenderMode};

/// TODO
pub trait Env {
    /// TODO
    type Action;
    /// TODO
    type Observation;
    /// TODO
    type Info;

    /// TODO
    type Metadata;
    /// TODO
    type ActionSpace;
    /// TODO
    type ObservationSpace;

    /// TODO
    fn step(&mut self, action: Self::Action) -> ActionReward<Self::Observation, Self::Info>;

    /// TODO
    fn reset(&mut self) -> Self::Observation;

    /// TODO
    fn render(&mut self, mode: RenderMode) -> Render;

    fn close(&mut self) {}

    /// TODO
    fn seed(&mut self, seed: Option<u64>) -> u64;

    fn rand_random(&self) -> &Pcg64;

    fn metadata(&self) -> &Self::Metadata;

    fn render_mode(&self) -> &RenderMode {
        DEFAULT_RENDER_MODE
    }

    fn reward_range(&self) -> &RewardRange {
        DEFAULT_REWARD_RANGE
    }

    fn action_space(&self) -> &Self::ActionSpace;
    fn observation_space(&self) -> &Self::ObservationSpace;
}

const DEFAULT_REWARD_RANGE: &'static RewardRange = &(RewardRange {
    lower_bound: f64::NEG_INFINITY,
    upper_bound: f64::INFINITY,
});

const DEFAULT_RENDER_MODE: &'static RenderMode = &RenderMode::None;

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

#[derive(Clone, Debug, Serialize)]
pub struct RewardRange {
    lower_bound: f64,
    upper_bound: f64,
}

impl Default for RewardRange {
    fn default() -> Self {
        DEFAULT_REWARD_RANGE.clone()
    }
}
