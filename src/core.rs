use std::fmt::Debug;

use ordered_float::OrderedFloat;
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::utils::{
    definitions::O64,
    renderer::{Render, RenderMode},
};

/// TODO
pub trait Env: Clone + PartialEq + Eq + Ord + Debug + Serialize
where
    Self::Observation: Into<Vec<f64>>,
{
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

    fn close(&mut self);

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
    lower_bound: OrderedFloat(f64::NEG_INFINITY),
    upper_bound: OrderedFloat(f64::INFINITY),
});

const DEFAULT_RENDER_MODE: &'static RenderMode = &RenderMode::None;

/// TODO
#[derive(Clone, Debug, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct ActionReward<T, E> {
    // TODO
    pub observation: T,
    // TODO
    pub reward: O64,
    // TODO
    pub done: bool,
    // TODO
    pub info: Option<E>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Ord, PartialOrd, Eq)]
pub struct RewardRange {
    lower_bound: O64,
    upper_bound: O64,
}

impl Default for RewardRange {
    fn default() -> Self {
        DEFAULT_REWARD_RANGE.clone()
    }
}
