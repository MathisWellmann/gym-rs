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

/// Defines the range of values that can be outputted by a given environment.
const DEFAULT_REWARD_RANGE: &'static RewardRange = &(RewardRange {
    lower_bound: OrderedFloat(f64::NEG_INFINITY),
    upper_bound: OrderedFloat(f64::INFINITY),
});

/// Defines the render mode set by a default environment instances.
const DEFAULT_RENDER_MODE: &'static RenderMode = &RenderMode::None;

/// Defines a common set of operations available to different environments.
pub trait Env: Clone + Debug + Serialize + EnvProperties
where
    Self::Observation: Sample + Into<Vec<f64>>,
{
    /// The type of action supported.
    type Action;

    /// The type of the observation produced after an action has been applied.
    type Observation;

    /// The type of the metadata object produced by acting on the environment.
    type Info;

    /// The type of the object produced when an environment is reset.
    type ResetInfo;

    /// Acts on an environment using the given action, producing a reward.
    fn step(&mut self, action: Self::Action) -> ActionReward<Self::Observation, Self::Info>;

    /// Resets the environment to a initial random state.
    fn reset(
        &mut self,
        seed: Option<u64>,
        return_info: bool,
        options: Option<BoxR<Self::Observation>>,
    ) -> (Self::Observation, Option<Self::ResetInfo>);

    /// Produces the renders, if any, associated with the given mode.
    fn render(&mut self, mode: RenderMode) -> Renders;

    /// Closes any open resources associated with the internal rendering service.
    fn close(&mut self);
}

/// Defines a set of properties that should be accessible in all environments.
pub trait EnvProperties
where
    Self: Sized,
{
    /// The type of values that can be observed in the action space.
    type ActionSpace;
    /// The type of observations produced
    type ObservationSpace;

    /// Provides an object describing additional details about this environment.
    fn metadata(&self) -> &Metadata<Self>;

    /// Provides the random number generator responsible for seeding states.
    fn rand_random(&self) -> &Pcg64;

    /// Provides the current render mode.
    fn render_mode(&self) -> &RenderMode {
        DEFAULT_RENDER_MODE
    }

    /// Provides the range of reward values that can be outputted by this environment.
    fn reward_range(&self) -> &RewardRange {
        DEFAULT_REWARD_RANGE
    }

    /// Provides the object describing the actions that can be observed.
    fn action_space(&self) -> &Self::ActionSpace;

    /// Provides the object describing the states that can be observed in this environment.
    fn observation_space(&self) -> &Self::ObservationSpace;
}

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
