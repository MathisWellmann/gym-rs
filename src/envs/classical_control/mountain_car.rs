use crate::core::{ActionReward, Env};
use crate::spaces::{self, Discrete, Space};
use crate::utils::math_ops;
use crate::utils::renderer::{RenderMode, Renderer};
use crate::utils::seeding::rand_random;
use derivative::Derivative;
use derive_new::new;
use rand::distributions::Uniform;
use rand::Rng;
use rand_pcg::Pcg64;
use serde::Serialize;

///Description:
///
///  The agent (a car) is started at the bottom of a valley. For any given
///  state, the agent may choose to accelerate to the left, right or cease
///  any acceleration.
///
///Source:
///
///  The environment appeared first in Andrew Moore's PhD Thesis (1990).
///  the source code in python: https://www.github.com/openai/gym
///
///Observation:
///
///  Num     Observation         Min     Max
///  0       Car Position        -1.2    0.6
///  1       Car Velocity        -0.07   0.07
///
///Actions:
///
///  In case of discrete action:
///  Num    Action
///  0      Accelerate to the left
///  1      Don't accelerate
///  2      Accelerate to the right
///
///  In case of continuous action, apply a force in range [-1.0, 1.0],
///  1.0 being maximum acceleration to the right,
///  -1.0 being maximum acceleration to the left
///
///  Note: This does not affect the amount of velocity affected by the gravitational pull acting on the cart.
///
///Reward:
///
///  Reward of 0 is awarded if the agent reached the flag (position = 0.5)
///  on top of the mountain.
///  Reward of -1 is awarded if the position of the agent is less than 0.5.
///
///Starting State:
///
///  The position of the car is assigned a uniform random value in
///  [-0.6, -0.4].
///  The starting velocity of the car is always assigned to 0.
///
///Episode Termination:
///
///  The car position is more than 0.5
///  Episode length is greater than 200
#[derive(Serialize, Derivative)]
#[derivative(Debug)]
pub struct MountainCarEnv<'a> {
    /// TODO
    pub min_position: f64,
    /// TODO
    pub max_position: f64,
    /// TODO
    pub max_speed: f64,
    /// TODO
    pub goal_position: f64,
    /// TODO
    pub goal_velocity: f64,

    /// TODO
    pub force: f64,
    /// TODO
    pub gravity: f64,

    /// TODO
    pub low: Observation,
    /// TODO
    pub high: Observation,

    /// TODO
    pub render_mode: RenderMode,
    /// TODO
    pub renderer: Renderer<'a>,

    // NOTE: Consider using SDL2 to reduce differences between gym_rs and the python implementation.
    /// TODO
    pub screen_width: u32,
    /// TODO
    pub screen_height: u32,
    /// TODO
    #[serde(skip_serializing)]
    #[derivative(Debug = "ignore")]
    pub screen: Option<sdl2::video::Window>,
    /// TODO
    #[serde(skip_serializing)]
    #[derivative(Debug = "ignore")]
    pub clock: Option<sdl2::TimerSubsystem>,
    #[serde(skip_serializing)]
    #[derivative(Debug = "ignore")]
    pub surf: Option<sdl2::surface::Surface<'a>>,
    /// TODO
    pub isopen: bool,

    /// TODO
    pub action_space: spaces::Discrete,
    /// TODO
    pub observation_space: spaces::Box<Observation>,

    /// TODO
    pub state: Observation,

    /// RANDOM NUMBER GENERATOR
    #[serde(skip_serializing)]
    rand_random: Pcg64,
    metadata: MountainCarMetadata,
}

const MOUNTAIN_CAR_RENDER_MODES: &'static [RenderMode] = &[
    RenderMode::Human,
    RenderMode::RgbArray,
    RenderMode::SingleRgbArray,
];

#[derive(Debug, Clone, Serialize)]
pub struct MountainCarMetadata {
    render_modes: &'static [RenderMode],
    render_fps: usize,
}

impl Default for MountainCarMetadata {
    fn default() -> Self {
        Self {
            render_modes: MOUNTAIN_CAR_RENDER_MODES,
            render_fps: 30,
        }
    }
}

/// Utility structure intended to reduce confusion around meaning of properties.
#[derive(Debug, new, Copy, Clone, Serialize)]
pub struct Observation(f64, f64);

impl Default for Observation {
    fn default() -> Self {
        Observation(0., 0.)
    }
}

impl Observation {
    /// TODO
    pub fn get_position(&self) -> f64 {
        self.0
    }

    /// TODO
    pub fn get_velocity(&self) -> f64 {
        self.1
    }

    /// TODO
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

impl<'a> MountainCarEnv<'a> {
    pub fn new(render_mode: RenderMode, goal_velocity: Option<f64>) -> Self {
        let (rand_random, _) = rand_random(None);

        let min_position = -1.2;
        let max_position = 0.6;
        let max_speed = 0.07;
        let goal_position = 0.5;
        let goal_velocity = goal_velocity.unwrap_or(0.);

        let force = 0.001;
        let gravity = 0.0025;

        let low = Observation(min_position, -max_speed);
        let high = Observation(max_position, max_speed);

        let renderer = Renderer::new(render_mode, None, None);

        // NOTE: Since rust requires statically typed properties, state must explicitly initiated or lazy
        // loaded via function (the later would deviate more from the current interface, so we
        // shouldn't use it).
        let state = Observation::default();

        let screen_width = 600;
        let screen_height = 400;
        let screen = None;
        let isopen = false;

        let action_space = spaces::Discrete(3);
        let observation_space = spaces::Box::new(low, high);

        let surf = None;
        let clock = None;

        let metadata = MountainCarMetadata::default();

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

            render_mode,
            renderer,

            action_space,
            observation_space,

            state,
            rand_random,

            screen,
            surf,
            screen_width,
            screen_height,
            clock,
            isopen,

            metadata,
        }
    }
}

impl<'a> Env for MountainCarEnv<'a> {
    type Action = usize;
    type Observation = Observation;
    type Info = String;
    type Metadata = MountainCarMetadata;
    type ActionSpace = Discrete;
    type ObservationSpace = spaces::Box<Self::Observation>;

    fn step(&mut self, action: Self::Action) -> ActionReward<Self::Observation, Self::Info> {
        assert!(
            self.action_space.contains(action),
            "{} (usize) invalid",
            action
        );

        let mut position = self.state.get_position();
        let mut velocity = self.state.get_velocity();

        velocity +=
            (action as isize - 1) as f64 * self.force + (3.0 * position).cos() * (-self.gravity);
        velocity = math_ops::clip(velocity, -self.max_speed, self.max_speed);

        position += velocity;
        position = math_ops::clip(position, self.min_position, self.max_position);

        if position == self.min_position && velocity < 0.0 {
            velocity = 0.0;
        }

        let done: bool = position >= self.goal_position && velocity >= self.goal_velocity;
        let reward: f64 = -1.0;

        self.state.update(position, velocity);

        ActionReward {
            observation: self.state,
            reward,
            done,
            info: None,
        }
    }

    fn reset(&mut self) -> Vec<f64> {
        let initial_position = Uniform::new::<f64, f64>(-0.6, -0.4);
        self.state = Observation::new(self.rand_random.sample(initial_position), 0.0);
        self.state.into()
    }

    fn render(
        &mut self,
        mode: crate::utils::renderer::RenderMode,
    ) -> crate::utils::renderer::Render {
        assert!(self.metadata.render_modes.contains(&mode));

        if self.screen.is_none() {
            let sdl_context = sdl2::init().unwrap();

            if mode == RenderMode::Human {
                let video_subsystem = sdl_context.video().unwrap();

                let window = video_subsystem
                    .window("Mountain Car", self.screen_width, self.screen_height)
                    .position_centered()
                    .build()
                    .unwrap();

                self.screen = Some(window);
            } else {
            }
        }

        todo!()
    }

    fn seed(&mut self, seed: Option<u64>) -> u64 {
        let (new_rng, new_rng_seed) = rand_random(seed);
        self.rand_random = new_rng;
        new_rng_seed
    }

    fn rand_random(&self) -> &Pcg64 {
        &self.rand_random
    }

    fn metadata(&self) -> &Self::Metadata {
        &self.metadata
    }

    fn action_space(&self) -> &Self::ActionSpace {
        &self.action_space
    }

    fn observation_space(&self) -> &Self::ObservationSpace {
        &self.observation_space
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn mountain_car() {
        let mut mc = MountainCarEnv::new(RenderMode::None, None);
        let _state = mc.reset();

        let mut rng = thread_rng();
        let mut end: bool = false;
        let mut episode_length = 0;
        while !end {
            if episode_length > 200 {
                break;
            }
            let action = rng.gen_range(0, 3);
            let ActionReward { done, .. } = mc.step(action);
            episode_length += 1;
            end = done;
            println!("episode_length: {}", episode_length);
        }
    }
}
