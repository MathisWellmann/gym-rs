use std::fmt::Debug;

use ordered_float::OrderedFloat;
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    envs::classical_control::utils::MaybeParseResetBoundsOptions,
    utils::{
        custom::{Metadata, O64},
        renderer::{RenderMode, Renders},
    },
};

/// TODO
pub trait Env: Clone + PartialEq + Eq + Ord + Debug + Serialize
where
    Self::Observation: Into<Vec<f64>>,
{
    type Action;
    type Observation;
    type Info;
    type ActionSpace;
    type ObservationSpace;
    type ResetInfo;

    /// TODO
    fn step(&mut self, action: Self::Action) -> ActionReward<Self::Observation, Self::Info>;

    /// TODO
    fn reset(
        &mut self,
        seed: Option<u64>,
        return_info: bool,
        options: Option<MaybeParseResetBoundsOptions>,
    ) -> (Self::Observation, Option<Self::ResetInfo>);

    /// TODO
    fn render(&mut self, mode: RenderMode) -> Renders;

    /// TODO
    fn seed(&mut self, seed: Option<u64>) -> u64;

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
    pub truncated: bool,
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
