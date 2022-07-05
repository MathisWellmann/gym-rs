use crate::core::{GymEnv, RenderMode};
use crate::utils::scale;
use crate::{spaces, utils};
use plotters::prelude::*;
use rand::distributions::Uniform;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

/**
Description:
    The agent (a car) is started at the bottom of a valley. For any given
    state, the agent may choose to accelerate to the left, right or cease
    any acceleration.

Source:
    The environment appeared first in Andrew Moore's PhD Thesis (1990).
    the source code in python: https://www.github.com/openai/gym

Observation:
    Num     Observation         Min     Max
    0       Car Position        -1.2    0.6
    1       Car Velocity        -0.07   0.07

Actions:
    In case of discrete action:
    Num    Action
    0      Accelerate to the left
    1      Don't accelerate
    2      Accelerate to the right

    In case of continuous action, apply a force in range [-1.0, 1.0],
    1.0 being maximum acceleration to the right,
    -1.0 being maximum acceleration to the left

    Note: This does not affect the amount of velocity affected by the gravitational pull acting on the cart.

Reward:
    Reward of 0 is awarded if the agent reached the flag (position = 0.5)
    on top of the mountain.
    Reward of -1 is awarded if the position of the agent is less than 0.5.

Starting State:
    The position of the car is assigned a uniform random value in
    [-0.6, -0.4].
    The starting velocity of the car is always assigned to 0.

Episode Termination:
    The car position is more than 0.5
    Episode length is greater than 200
**/

#[derive(Debug)]
pub struct MountainCarEnv {
    pub min_position: f64,
    pub max_position: f64,
    pub max_speed: f64,
    pub goal_position: f64,
    pub goal_velocity: f64,

    pub force: f64,
    pub gravity: f64,

    pub low: Observation,
    pub high: Observation,

    // NOTE: Consider using SDL2 to reduce differences between gym_rs and the python implementation.
    pub screen_width: usize,
    pub screen_height: usize,
    pub screen: Option<f32>,
    // number of episodes
    pub clock: usize,
    pub isopen: bool,

    pub action_space: spaces::Discrete,
    pub observation_space: spaces::Box<Observation>,

    pub state: Observation,
    /// RANDOM NUMBER GENERATOR
    rng: Pcg64,
}

// Utility structure intended to reduce confusion around meaning of properties.
#[derive(Debug)]
pub struct Observation(f64, f64);

impl Default for Observation {
    fn default() -> Self {
        Observation(0., 0.)
    }
}

impl Observation {
    pub fn get_position(&self) -> f64 {
        self.0
    }

    pub fn get_velocity(&self) -> f64 {
        self.1
    }

    pub fn update(&mut self, position: f64, velocity: f64) {
        self.0 = position;
        self.1 = velocity;
    }
}

impl From<Observation> for Vec<f64> {
    fn from(observation: Observation) -> Self {
        vec![observation.0, observation.1]
    }
}

impl MountainCarEnv {
    fn new(render_mode: Option<&str>, goal_velocity: Option<f64>) -> Self {
        let rng = Pcg64::from_entropy();

        let min_position = -1.2;
        let max_position = 0.6;
        let max_speed = 0.07;
        let goal_position = 0.5;
        let goal_velocity = goal_velocity.unwrap_or(0.);

        let force = 0.001;
        let gravity = 0.0025;

        let low = Observation(min_position, -max_speed);
        let high = Observation(max_position, max_speed);

        let render_mode = "";
        let renderer = "";

        // NOTE: Since rust requires statically typed properties, state must explicitly initiated or lazy
        // loaded via function (the later would deviate more from the current interface, so we
        // shouldn't use it).
        let state = Observation::default();

        let clock = 0;
        let screen_width = 600;
        let screen_height = 400;

        let action_space = spaces::Discrete(3);
        let observation_space = spaces::Box(low, high);

        Self {
            min_position,
            max_position,
            max_speed,
            goal_position,
            goal_velocity,

            force,
            gravity,

            low,
            high,

            action_space,
            observation_space,

            state,
            rng,

            screen_width,
            screen_height,
            clock,
            screen: todo!(),
            isopen: todo!(),
        }
    }
}

impl GymEnv for MountainCarEnv {
    type Action = usize;

    fn step(&mut self, action: Self::Action) -> (Vec<f64>, f64, bool, Option<String>) {
        assert!(
            self.action_space.contains(action),
            "{} (usize) invalid",
            action
        );

        let mut position = self.state.get_position();
        let mut velocity = self.state.get_velocity();

        velocity += (action - 1) as f64 * self.force + (3.0 * position).cos() * (-self.gravity);
        velocity = utils::clip(velocity, -self.max_speed, self.max_speed);

        position += velocity;
        position = utils::clip(position, self.min_position, self.max_position);

        if position == self.min_position && velocity < 0.0 {
            velocity = 0.0;
        }

        let done: bool = position >= self.goal_position && velocity >= self.goal_velocity;
        let reward: f64 = -1.0;

        self.state.update(position, velocity);

        (self.state.into(), reward, done, None)
    }

    fn reset(&mut self) -> Vec<f64> {
        let d = Uniform::new(-0.6, -0.4);
        self.state = [self.rng.sample(d), 0.0];
        self.state.to_vec()
    }

    /// render the environment using 30 frames per second
    fn render(&self, mode: RenderMode) {}

    fn seed(&mut self, seed: u64) {
        self.rng = Pcg64::seed_from_u64(seed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn mountain_car() {
        let mut mc = MountainCarEnv::new(None, None);
        let _state = mc.reset();

        let mut rng = thread_rng();
        let mut end: bool = false;
        while !end {
            if mc.episode_length > 200 {
                break;
            }
            let action = rng.gen_range(0, 3);
            let (_state, _r, done, _) = mc.step(action);
            end = done;
            println!("episode_length: {}", mc.episode_length);
        }
    }
}
