use core::fmt;
use rand_pcg::Pcg64;
use serde::{Serialize, de::DeserializeOwned};

use crate::{spaces::BoxR, utils::custom::structs::Metadata};

/// Defines the range of values that can be outputted by a given environment.
const DEFAULT_REWARD_RANGE: &'static RewardRange = &(RewardRange {
    lower_bound: f64::NEG_INFINITY,
    upper_bound: f64::INFINITY,
});

/// Defines a common set of operations available to different environments.
pub trait Env: Clone + fmt::Debug + Serialize + EnvProperties {
    /// The type of the metadata object produced by acting on the environment.
    type Info;

    /// The type of the object produced when an environment is reset.
    type ResetInfo;

    /// Generate an instance.
    fn new() -> Self;

    /// Acts on an environment using the given action, producing a reward.
    fn step(&mut self, action: usize) -> ActionReward<Self::Observation, Self::Info>;

    /// Resets the environment to a initial random state.
    fn reset(
        &mut self,
        seed: Option<u64>,
        return_info: bool,
        options: Option<BoxR<Self::Observation>>,
    ) -> (Self::Observation, Option<Self::ResetInfo>);

    /// Closes any open resources associated with the internal rendering service.
    fn close(&mut self);
}

/// Defines a set of properties that should be accessible in all environments.
pub trait EnvProperties
where
    Self: Sized,
    Self::Observation: Copy + Serialize + DeserializeOwned + Into<Vec<f64>>
{
    /// The type of values that can be observed in the action space.
    type ActionSpace;
    /// The type of observations produced
    type ObservationSpace;
    /// The state value.
    type Observation;

    /// The default score when episode terminates due to expected errors.
    const DEFAULT_SCORE: f64;

    /// The length of an episode.
    fn episode_length() -> usize;

    /// Provides an object describing additional details about this environment.
    fn metadata(&self) -> &Metadata<Self>;

    /// Provides the random number generator responsible for seeding states.
    fn rand_random(&self) -> &Pcg64;

    /// Provides the range of reward values that can be outputted by this environment.
    fn reward_range(&self) -> &RewardRange {
        DEFAULT_REWARD_RANGE
    }

    /// Provides the object describing the actions that can be observed.
    fn action_space(&self) -> &Self::ActionSpace;

    /// Provides the object describing the states that can be observed in this environment.
    fn observation_space(&self) -> &Self::ObservationSpace;

    /// Set state.
    fn set_observation(&mut self, state: Self::Observation);

    /// Get partial state.
    fn get_observation_property(&self, idx: usize) -> f64;

    /// Get full state.
    fn get_observation(&self) -> Self::Observation;
}

/// Encapsulates and describes the state update experienced by an environment after acting on an
/// action.
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub struct ActionReward<T, E> {
    /// The current observable state.
    pub observation: T,
    /// The value of the reward produced.
    pub reward: f64,
    /// Indicates whether the episode has terminated or not.
    pub done: bool,
    /// Indicates whether the episode has termianted early or  not.
    pub truncated: bool,
    /// Additional info implementations may provide for purposes beyond classical RL.
    pub info: Option<E>,
}

/// Defines the bounds for the reward value that can be observed.
#[derive(Clone, Debug, Serialize, PartialEq, PartialOrd)]
pub struct RewardRange {
    /// The smallest possible reward that can be observed.
    lower_bound: f64,
    /// The largest possible reward that can be observed.
    upper_bound: f64,
}

/// Implement a default reward range.
impl Default for RewardRange {
    fn default() -> Self {
        DEFAULT_REWARD_RANGE.clone()
    }
}
