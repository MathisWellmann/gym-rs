use std::fmt::Debug;

use ordered_float::OrderedFloat;
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    spaces::BoxR,
    utils::{
        custom::{Metadata, Sample, O64},
        renderer::{RenderMode, Renders},
    },
};

/// Defines a common set of operations available to different environments.
pub trait Env: Clone + Debug + Serialize
where
    Self::Observation: Sample + Into<Vec<f64>>,
{
    /// The type of action supported.
    type Action;
    /// The type of the observation produced after an action has been applied.
    type Observation;
    /// The type of the metadata object produced by acting on the environment.
    type Info;
    /// The type of values that can be observed in the action space.
    type ActionSpace;
    /// The type of observations produced
    type ObservationSpace;
    /// The type of the object produced when an environment is reset.
    type ResetInfo;

    /// Acts on an environment using the given action, producing the rewarded and additional
    /// context. 
    ///
    /// Examples:
    ///     
    fn step(&mut self, action: Self::Action) -> ActionReward<Self::Observation, Self::Info>;

    fn reset(
        &mut self,
        seed: Option<u64>,
        return_info: bool,
        options: Option<BoxR<Self::Observation>>,
    ) -> (Self::Observation, Option<Self::ResetInfo>);

    fn render(&mut self, mode: RenderMode) -> Renders;

    fn close(&mut self);

    // Properties
    fn metadata(&self) -> &Metadata<Self>;
    fn rand_random(&self) -> &Pcg64;
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

#[derive(Clone, Debug, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct ActionReward<T, E> {
    pub observation: T,
    pub reward: O64,
    pub done: bool,
    pub truncated: bool,
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
