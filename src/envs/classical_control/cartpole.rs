use log::warn;

use std::{f64::consts::PI, ops::Neg};

use derive_new::new;
use rand::{
    distributions::{
        uniform::{SampleUniform, UniformFloat, UniformSampler},
        Uniform,
    },
    prelude::Distribution,
    Rng,
};

use rand_pcg::Pcg64;
use serde::{Serialize, Deserialize};

use crate::{
    core::{ActionReward, Env, EnvProperties},
    spaces::{BoxR, Discrete, Space},
    utils::{
        custom::{structs::Metadata, traits::Sample},
        seeding::{self, rand_random},
    },
};

/// An environment which implements the cart pole problem described in
/// [Neuronlike adaptive elements that can solve difficult learning control
/// problems](https://ieeexplore.ieee.org/document/6313077).
///
/// The problem involves applying the correct forces onto a cart with a pole hinged onto it,
/// in order to ensure the pole remains within the preconfigured regions.
/// The agent starts by being assigned random values between (-0.05, 0.05) for all
/// fields available in the state structure. The agent is rewarded '+1' for every step taken until the episode ends.
///
/// The episode ends when any of the following conditions occur:
///
/// 1. Termination: [`CartPoleObservation.theta`] is greater than +/-12.0 (pole has fallen).
/// 2. Termination: [`CartPoleObservation.x`] is greater than +/-2.4 (cart is outside bounds).
/// 3. Truncation: Episode length is greater than 500.
#[derive(Debug, Clone, Serialize)]
pub struct CartPoleEnv {
    /// The available actions that can be taken.
    pub action_space: Discrete,
    /// The range of values that can be observed.
    pub observation_space: BoxR<CartPoleObservation>,
    /// The current state of the environment.
    pub state: CartPoleObservation,
    /// Additional pieces of information provided by the environment.
    pub metadata: Metadata<Self>,
    /// The gravity constant applied to the environment..
    pub gravity: f64,
    /// The mass of the cart.
    pub masscart: f64,
    /// The mass of the pole.
    pub masspole: f64,
    /// Half the length of the pole.
    pub length: f64,
    /// The default force applied to the pole.
    pub force_mag: f64,
    /// The number of seconds between state updates.
    pub tau: f64,
    /// The type of integration done on the differential equations found in the paper.
    pub kinematics_integrator: KinematicsIntegrator,
    /// The angle that the pole can lean to before an episode is considered terminated.
    pub theta_threshold_radians: f64,
    /// The x value that the cart can be at before an episode is considered terminated.
    pub x_threshold: f64,
    /// The number of steps taken after the episode was terminated.
    pub steps_beyond_terminated: Option<usize>,
    #[serde(skip_serializing)]
    rand_random: Pcg64,
}

impl CartPoleEnv {
    fn total_mass(&self) -> f64 {
        self.masspole + self.masscart
    }

    fn polemass_length(&self) -> f64 {
        self.masspole + self.length
    }
}

impl Default for Metadata<CartPoleEnv> {
    fn default() -> Self {
        Metadata::new(&[], 50)
    }
}

/// The sampler responsible for generating an observation using uniform probability.
pub struct UniformCartPoleObservation {
    x_sampler: UniformFloat<f64>,
    x_dot_sampler: UniformFloat<f64>,
    theta_sampler: UniformFloat<f64>,
    theta_dot_sampler: UniformFloat<f64>,
}

impl SampleUniform for CartPoleObservation {
    type Sampler = UniformCartPoleObservation;
}

impl UniformSampler for UniformCartPoleObservation {
    type X = CartPoleObservation;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
    {
        UniformCartPoleObservation {
            x_sampler: UniformFloat::new(low.borrow().x, high.borrow().x),
            x_dot_sampler: UniformFloat::new(low.borrow().x_dot, high.borrow().x_dot),
            theta_sampler: UniformFloat::new(low.borrow().theta, high.borrow().theta),
            theta_dot_sampler: UniformFloat::new(low.borrow().theta_dot, high.borrow().theta_dot),
        }
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
    {
        UniformCartPoleObservation {
            x_sampler: UniformFloat::new_inclusive(low.borrow().x, high.borrow().x),
            x_dot_sampler: UniformFloat::new_inclusive(low.borrow().x_dot, high.borrow().x_dot),
            theta_sampler: UniformFloat::new_inclusive(low.borrow().theta, high.borrow().theta),
            theta_dot_sampler: UniformFloat::new_inclusive(
                low.borrow().theta_dot,
                high.borrow().theta_dot,
            ),
        }
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        CartPoleObservation {
            x: self.x_sampler.sample(rng),
            x_dot: self.x_dot_sampler.sample(rng),
            theta: self.theta_sampler.sample(rng),
            theta_dot: self.theta_dot_sampler.sample(rng),
        }
    }
}

/// Defines the state found in the cart pole environment.
#[derive(new, Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub struct CartPoleObservation {
    x: f64,
    x_dot: f64,
    theta: f64,
    theta_dot: f64,
}

impl From<CartPoleObservation> for Vec<f64> {
    fn from(value: CartPoleObservation) -> Self {
        vec![value.x, value.x_dot, value.theta, value.theta_dot]
    }
}

impl Sample for CartPoleObservation {
    fn sample_between<R: Rng>(rng: &mut R, bounds: Option<BoxR<Self>>) -> Self {
        let BoxR { low, high } = bounds.unwrap_or({
            let observation_bound = CartPoleObservation::new(0.5, 0.5, 0.5, 0.5);
            BoxR::new(-observation_bound, observation_bound)
        });

        Uniform::new(low, high).sample(rng)
    }
}

impl Neg for CartPoleObservation {
    type Output = CartPoleObservation;

    fn neg(self) -> Self::Output {
        CartPoleObservation {
            x: -self.x,
            x_dot: -self.x_dot,
            theta: -self.theta,
            theta_dot: -self.theta_dot,
        }
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
/// Describes the available types of integration on cartpole equations.
pub enum KinematicsIntegrator {
    /// Euler integration.
    Euler,
    /// Semi-implicit Euler integration.
    Other,
}

impl Env for CartPoleEnv {
    type Info = ();

    type ResetInfo = ();

    /// Creates a cart pole environment using defaults from the paper.
    fn new() -> Self {
        let (mut rand_random, _) = rand_random(None);

        let gravity = 9.8;
        let masscart = 1.0;
        let masspole = 0.1;
        let length = 0.5;
        let force_mag = 10.0;
        let tau = 0.02;
        let kinematics_integrator = KinematicsIntegrator::Euler;

        let theta_threshold_radians = 12. * 2. * PI / 360.;
        let x_threshold = 2.4;

        let high = CartPoleObservation::new(
            x_threshold * 2.,
            f64::INFINITY,
            theta_threshold_radians * 2.,
            f64::INFINITY,
        );

        let action_space = Discrete(2);
        let observation_space = BoxR::new(-high, high);

        let metadata = Metadata::default();

        let state = CartPoleObservation::sample_between(&mut rand_random, None);

        let steps_beyond_terminated = None;

        Self {
            gravity,
            masscart,
            masspole,
            length,
            force_mag,
            tau,
            kinematics_integrator,
            theta_threshold_radians,
            x_threshold,
            action_space,
            observation_space,
            state,
            metadata,
            rand_random,
            steps_beyond_terminated,
        }
    }

    fn step(&mut self, action: usize) -> crate::core::ActionReward<Self::Observation, Self::Info> {
        assert!(
            self.action_space.contains(action),
            "{} usize invalid",
            action
        );

        let CartPoleObservation {
            mut x,
            mut x_dot,
            mut theta,
            mut theta_dot,
        } = self.state;
        let force = if action == 1 {
            self.force_mag
        } else {
            -self.force_mag
        };

        let costheta = theta.cos();
        let sintheta = theta.sin();

        let temp =
            (force + self.polemass_length() * theta_dot.powf(2.) * sintheta) / self.total_mass();
        let thetaacc = (self.gravity * sintheta - costheta * temp)
            / (self.length * ((4.0 / 3.0) - self.masspole * costheta.powf(2.) / self.total_mass()));
        let xacc = temp - self.polemass_length() * thetaacc * costheta / self.total_mass();

        if self.kinematics_integrator == KinematicsIntegrator::Euler {
            x = x + self.tau * x_dot;
            x_dot = x_dot + self.tau * xacc;
            theta = theta + self.tau * theta_dot;
            theta_dot = theta_dot + self.tau * thetaacc;
        } else {
            x_dot = x_dot + self.tau * xacc;
            x = x + self.tau * x_dot;
            theta_dot = theta_dot + self.tau * thetaacc;
            theta = theta + self.tau * theta_dot;
        }

        self.state = CartPoleObservation {
            x,
            x_dot,
            theta_dot,
            theta,
        };

        let done = x < -self.x_threshold
            || x > self.x_threshold
            || theta < -self.theta_threshold_radians
            || theta > self.theta_threshold_radians;

        let reward = if !done {
            1.0
        } else if self.steps_beyond_terminated.is_none() {
            self.steps_beyond_terminated = Some(0);
            1.0
        } else {
            warn!("Calling step after termination may result in undefined behaviour. Consider reseting.");
            self.steps_beyond_terminated = self.steps_beyond_terminated.map(|step| step + 1);
            0.
        };

        let _metadata = &self.metadata;
        let _x_threshold = self.x_threshold;
        let _length = self.length;
        let _state = self.state;

        ActionReward {
            observation: self.state,
            reward,
            done,
            truncated: false,
            info: Some(()),
        }
    }

    fn reset(
        &mut self,
        seed: Option<u64>,
        return_info: bool,
        options: Option<BoxR<Self::Observation>>,
    ) -> (Self::Observation, Option<Self::ResetInfo>) {
        let (rand_random, _) = seeding::rand_random(seed);
        self.rand_random = rand_random;

        self.state = CartPoleObservation::sample_between(&mut self.rand_random, options);

        let _metadata = &self.metadata;
        let _x_threshold = self.x_threshold;
        let _length = self.length;
        let _state = self.state;

        self.steps_beyond_terminated = None;

        if return_info {
            (self.state, Some(()))
        } else {
            (self.state, None)
        }
    }

    fn close(&mut self) {}
}

impl EnvProperties for CartPoleEnv {
    type ActionSpace = Discrete;

    type ObservationSpace = BoxR<CartPoleObservation>;

    type Observation = CartPoleObservation;

    fn set_observation(&mut self, state: Self::Observation) {
        self.state = state
    }

    fn get_observation(&self) -> Self::Observation {
        self.state
    }

    fn get_observation_property(&self, idx: usize) -> f64 {
        match idx {
            0 => self.state.x,
            1 => self.state.x_dot,
            2 => self.state.theta,
            3 => self.state.theta_dot,
            _ => unreachable!("Wrong idx."),
        }
    }

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

    fn episode_length() -> usize {
        Self::DEFAULT_SCORE as usize
    }

    const DEFAULT_SCORE: f64 = 500.;
}
