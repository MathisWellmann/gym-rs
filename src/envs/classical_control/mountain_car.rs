use num_traits::Float;
use ordered_float::impl_rand::UniformOrdered;
use rand::distributions::uniform::{SampleUniform, UniformSampler};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::Rng;
use std::fmt::Debug;

use crate::core::{ActionReward, Env, EnvProperties};
use crate::spaces::{self, BoxR, Discrete, Space};

use crate::utils::custom::structs::Metadata;
use crate::utils::custom::traits::Sample;
use crate::utils::custom::types::O64;
use crate::utils::custom::util_fns::clip;
use crate::utils::seeding::rand_random;
use derivative::Derivative;
use derive_new::new;

use ordered_float::OrderedFloat;
use rand_pcg::Pcg64;

use serde::Serialize;

/// An implementation of the classical reinforcment learning environment, mountain car.
///
/// The problem involves moving a stochastically placed car on the bottom of a sinusoidal valley
/// to the top of the hill by applying left and right forces to the car. The car is given a reward
/// of `-1` for each step taken by the agent, until a terminal step is reached.
///
/// An episode ends when one of the following conditions occur:
///     1. Termination: The car reaches the goal position.
///     2. Truncation: The episode exceeds 200 steps.
#[derive(Serialize, Derivative)]
#[derivative(Debug)]
pub struct MountainCarEnv {
    /// The minimum position the car can be spawned at.
    pub min_position: O64,
    /// The maximum position the cart can be spawned at.
    pub max_position: O64,
    /// The max speed the car can reach.
    pub max_speed: O64,
    /// The position on the map, where when passed, an episode can be considered terminated.
    pub goal_position: O64,
    /// The velocity at which an episode can be considered terminated.
    pub goal_velocity: O64,

    /// The force of the cart.
    pub force: O64,
    /// The gravity constant applied to the environment.
    pub gravity: O64,

    /// The set of actions which can be taken.
    pub action_space: spaces::Discrete,
    /// The range of values that can be observed.
    pub observation_space: spaces::BoxR<MountainCarObservation>,

    /// The state of the environment.
    pub state: MountainCarObservation,

    /// Additional information provided by the environment.
    pub metadata: Metadata<Self>,

    #[serde(skip_serializing)]
    #[derivative(Debug = "ignore")]
    rand_random: Pcg64,
}

impl Clone for MountainCarEnv {
    fn clone(&self) -> Self {
        Self {
            min_position: self.min_position.clone(),
            max_position: self.max_position.clone(),
            max_speed: self.max_speed.clone(),
            goal_position: self.goal_position.clone(),
            goal_velocity: self.goal_velocity.clone(),
            force: self.force.clone(),
            gravity: self.gravity.clone(),
            action_space: self.action_space.clone(),
            observation_space: self.observation_space.clone(),
            state: self.state.clone(),
            rand_random: self.rand_random.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Default for Metadata<MountainCarEnv> {
    fn default() -> Self {
        Metadata::new(&[], 30)
    }
}

/// Utility structure intended to reduce confusion around meaning of properties.
#[derive(Debug, new, Copy, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct MountainCarObservation {
    /// The position the car exists on the mountain.
    pub position: O64,
    /// The velocity the car is travelling at.
    pub velocity: O64,
}

/// The structure responsible for uniformly sampling a mountain car observation.
pub struct UniformMountainCarObservation {
    /// The sampler responsible for deriving a position.
    pub position_sampler: UniformOrdered<f64>,
}

impl UniformSampler for UniformMountainCarObservation {
    type X = MountainCarObservation;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
    {
        UniformMountainCarObservation {
            position_sampler: UniformOrdered::new(low.borrow().position, high.borrow().position),
        }
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
    {
        UniformMountainCarObservation {
            position_sampler: UniformOrdered::new_inclusive(
                low.borrow().position,
                high.borrow().position,
            ),
        }
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        MountainCarObservation {
            position: self.position_sampler.sample(rng),
            velocity: OrderedFloat(0.),
        }
    }
}

impl SampleUniform for MountainCarObservation {
    type Sampler = UniformMountainCarObservation;
}

impl Sample for MountainCarObservation {
    fn sample_between<R: Rng>(rng: &mut R, bounds: Option<BoxR<Self>>) -> Self {
        let BoxR { low, high } = bounds.unwrap_or({
            BoxR::new(
                MountainCarObservation {
                    position: OrderedFloat(-0.6),
                    velocity: OrderedFloat(0.),
                },
                MountainCarObservation {
                    position: OrderedFloat(-0.4),
                    velocity: OrderedFloat(0.),
                },
            )
        });

        Uniform::new(low, high).sample(rng)
    }
}

impl From<MountainCarObservation> for Vec<f64> {
    fn from(o: MountainCarObservation) -> Self {
        vec![o.position.into_inner(), o.velocity.into_inner()]
    }
}

impl MountainCarEnv {
    fn height(xs: &Vec<O64>) -> Vec<O64> {
        xs.clone()
            .iter()
            .map(|value| OrderedFloat((3. * value.into_inner()).sin() * 0.45 + 0.55))
            .collect()
    }
}

impl Env for MountainCarEnv {
    type Action = usize;
    type Observation = MountainCarObservation;
    type Info = ();
    type ResetInfo = ();

    fn step(
        &mut self,
        action: Self::Action,
    ) -> ActionReward<<Self as Env>::Observation, Self::Info> {
        assert!(
            self.action_space.contains(action),
            "{} (usize) invalid",
            action
        );

        let mut position = self.state.position;
        let mut velocity = self.state.velocity;

        velocity += OrderedFloat((action as f64) - 1.) * self.force
            + (OrderedFloat(3.) * position).cos() * (-self.gravity);
        velocity = clip(velocity, -self.max_speed, self.max_speed);

        position += velocity;
        position = clip(position, self.min_position, self.max_position);

        if position == self.min_position && velocity < OrderedFloat(0.) {
            velocity = OrderedFloat(0.);
        }

        let done: bool = position >= self.goal_position && velocity >= self.goal_velocity;
        let reward: O64 = OrderedFloat(-1.0);

        self.state = MountainCarObservation { position, velocity };

        ActionReward {
            observation: self.state,
            reward,
            done,
            truncated: false,
            info: None,
        }
    }

    fn reset(
        &mut self,
        seed: Option<u64>,
        return_info: bool,
        options: Option<BoxR<Self::Observation>>,
    ) -> (Self::Observation, Option<Self::ResetInfo>) {
        let (rand_random, _) = rand_random(seed);
        self.rand_random = rand_random;

        self.state = MountainCarObservation::sample_between(&mut self.rand_random, options);

        if return_info {
            (self.state, Some(()))
        } else {
            (self.state, None)
        }
    }

    fn close(&mut self) {}

    /// paper.
    fn new() -> Self {
        let (mut rng, _) = rand_random(None);

        let min_position = OrderedFloat(-1.2);
        let max_position = OrderedFloat(0.6);
        let max_speed = OrderedFloat(0.07);
        let goal_position = OrderedFloat(0.5);
        let goal_velocity = OrderedFloat(0.);

        let force = OrderedFloat(0.001);
        let gravity = OrderedFloat(0.0025);

        let low = MountainCarObservation::new(min_position, -max_speed);
        let high = MountainCarObservation::new(max_position, max_speed);

        let state = MountainCarObservation::sample_between(&mut rng, None);

        let metadata = Metadata::default();

        let action_space = spaces::Discrete(3);
        let observation_space = spaces::BoxR::new(low, high);

        Self {
            min_position,
            max_position,
            max_speed,
            goal_position,
            goal_velocity,

            force,
            gravity,

            action_space,
            observation_space,

            state,
            rand_random: rng,

            metadata,
        }
    }
}

impl EnvProperties for MountainCarEnv
where
    Self: Sized,
{
    type ActionSpace = Discrete;
    type ObservationSpace = spaces::BoxR<<Self as Env>::Observation>;

    fn metadata(&self) -> &Metadata<Self> {
        &self.metadata
    }

    fn rand_random(&self) -> &Pcg64 {
        &self.rand_random
    }

    fn action_space(&self) -> &Self::ActionSpace {
        &self.action_space
    }

    fn observation_space(&self) -> &Self::ObservationSpace {
        &self.observation_space
    }
}
