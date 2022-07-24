use num_traits::Float;
use ordered_float::impl_rand::UniformOrdered;
use rand::distributions::uniform::{SampleUniform, UniformSampler};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use sdl2::gfx::primitives::DrawRenderer;
use std::fmt::Debug;
use std::iter::zip;

use crate::core::{ActionReward, Env};
use crate::spaces::{self, BoxR, Discrete, Space};
use crate::utils::custom::{self, Metadata, Sample, Screen, ScreenGuiTransformations, O64};
use crate::utils::renderer::{RenderMode, Renderer, Renders};
use crate::utils::seeding::rand_random;
use derivative::Derivative;
use derive_new::new;
use na::{Point2, Rotation2};
use nalgebra as na;
use ordered_float::OrderedFloat;
use rand_pcg::Pcg64;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use serde::Serialize;

/// # Description:
///
///  The agent (a car) is started at the bottom of a valley. For any given
///  state, the agent may choose to accelerate to the left, right or cease
///  any acceleration.
///
/// # Source:
///
///  The environment appeared first in Andrew Moore's PhD Thesis (1990).
///  the source code in python: https://www.github.com/openai/gym
///
/// # Observation:
///
///  Num     Observation         Min     Max
///  0       Car Position        -1.2    0.6
///  1       Car Velocity        -0.07   0.07
///
/// # Actions:
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
/// # Reward:
///
///  Reward of 0 is awarded if the agent reached the flag (position = 0.5)
///  on top of the mountain.
///  Reward of -1 is awarded if the position of the agent is less than 0.5.
///
/// # Starting State:
///
///  The position of the car is assigned a uniform random value in
///  [-0.6, -0.4].
///  The starting velocity of the car is always assigned to 0.
///
/// # Episode Termination:
///
///  The car position is more than 0.5
///  Episode length is greater than 200
#[derive(Serialize, Derivative)]
#[derivative(Debug)]
pub struct MountainCarEnv {
    pub min_position: O64,
    pub max_position: O64,
    pub max_speed: O64,
    pub goal_position: O64,
    pub goal_velocity: O64,

    pub force: O64,
    pub gravity: O64,

    pub render_mode: RenderMode,
    pub renderer: Renderer,

    pub screen: Screen,

    pub action_space: spaces::Discrete,
    pub observation_space: spaces::BoxR<MountainCarObservation>,

    pub state: MountainCarObservation,

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
            render_mode: self.render_mode.clone(),
            renderer: self.renderer.clone(),
            screen: self.screen.clone(),
            action_space: self.action_space.clone(),
            observation_space: self.observation_space.clone(),
            state: self.state.clone(),
            rand_random: self.rand_random.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

const MOUNTAIN_CAR_RENDER_MODES: &'static [RenderMode] = &[
    RenderMode::Human,
    RenderMode::RgbArray,
    RenderMode::SingleRgbArray,
    RenderMode::None,
];

impl Default for Metadata<MountainCarEnv> {
    fn default() -> Self {
        Metadata::new(MOUNTAIN_CAR_RENDER_MODES, 30)
    }
}

/// Utility structure intended to reduce confusion around meaning of properties.
#[derive(Debug, new, Copy, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct MountainCarObservation {
    pub position: O64,
    pub velocity: O64,
}

pub struct UniformMountainCarObservation {
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
    fn sample_between(rng: &mut Pcg64, bounds: Option<BoxR<Self>>) -> Self {
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

    fn render(
        mode: RenderMode,
        max_position: O64,
        min_position: O64,
        goal_position: O64,
        state: MountainCarObservation,
        screen: &mut Screen,
        metadata: &Metadata<Self>,
    ) -> Renders {
        assert!(metadata.render_modes.contains(&mode));

        screen.load_gui();
        screen.consume_events();

        let world_width = max_position - min_position;
        let scale = OrderedFloat(screen.width as f64) / world_width;
        let carwidth = 40;
        let carheight = 20;

        screen.draw_on_canvas(
            |internal_canvas| {
                internal_canvas.set_draw_color(Color::WHITE);
                internal_canvas.clear();

                let pos = state.position;

                let xs: Vec<_> = (0..100)
                    .into_iter()
                    .map(|index| (((max_position - min_position) / 100.) * index as f64))
                    .map(|value| value + min_position)
                    .collect();

                let ys: Vec<_> = Self::height(&xs);
                let xys: Vec<Point> = zip(
                    xs.iter().map(|value| (value - min_position) * scale),
                    ys.iter().map(|value| value * scale),
                )
                .map(|(x, y)| {
                    Point::new(x.floor().into_inner() as i32, y.floor().into_inner() as i32)
                })
                .collect();

                internal_canvas.set_draw_color(Color::BLACK);
                internal_canvas.draw_lines(xys.as_slice()).unwrap();

                let clearance = 10f64;

                let (l, r, t, b) = (-carwidth / 2, carwidth / 2, carheight, 0);
                let coords = [(l, b), (l, t), (r, t), (r, b)].map(|(x, y)| {
                    let point = Point2::new(x as f64, y as f64);
                    let desired_angle = (OrderedFloat(3.) * pos).cos().into_inner();
                    let rotation_matrix = Rotation2::new(desired_angle);
                    let rotated_point = rotation_matrix.transform_point(&point);

                    let (x, y) = (rotated_point.x, rotated_point.y);

                    let new_x = OrderedFloat(x) + (pos - min_position) * scale;
                    let new_y = OrderedFloat(y)
                        + clearance
                        + Self::height(&vec![pos]).pop().unwrap() * scale;

                    (new_x, new_y)
                });

                let coords_x = coords.map(|coord| coord.0.floor().into_inner() as i16);
                let coords_y = coords.map(|coord| coord.1.floor().into_inner() as i16);

                internal_canvas
                    .aa_polygon(&coords_x, &coords_y, Color::BLACK)
                    .unwrap();

                internal_canvas
                    .filled_polygon(&coords_x, &coords_y, Color::BLACK)
                    .unwrap();

                for (x, y) in [(carwidth as f64 / 4., 0.), ((-carwidth as f64 / 4.), 0.)] {
                    let point = Point2::new(x as f64, y as f64);
                    let desired_angle = ((OrderedFloat(3.) * pos).cos()).into_inner();
                    let rotation_matrix = Rotation2::new(desired_angle);
                    let rotated_point = rotation_matrix.transform_point(&point);

                    let (x, y) = (OrderedFloat(rotated_point.x), OrderedFloat(rotated_point.y));

                    let (wheel_x, wheel_y) = (
                        (x + (pos - min_position) * scale).floor().into_inner() as i16,
                        (y + clearance + Self::height(&vec![pos]).pop().unwrap() * scale)
                            .floor()
                            .into_inner() as i16,
                    );

                    let rad = (carheight as f64 / 2.5).floor() as i16;

                    internal_canvas
                        .aa_circle(wheel_x, wheel_y, rad, Color::RGB(128, 128, 128))
                        .unwrap();

                    internal_canvas
                        .filled_circle(wheel_x, wheel_y, rad, Color::RGB(128, 128, 128))
                        .unwrap();
                }

                let flagx = ((goal_position - min_position) * scale)
                    .floor()
                    .into_inner() as i16;
                let flagy1 = (Self::height(&vec![goal_position]).pop().unwrap() * scale)
                    .floor()
                    .into_inner() as i16;
                let flagy2 = flagy1 + 50;
                internal_canvas
                    .vline(flagx, flagy1, flagy2, Color::BLACK)
                    .unwrap();

                internal_canvas
                    .aa_polygon(
                        &vec![flagx, flagx, flagx + 25],
                        &vec![flagy2, flagy2 - 10, flagy2 - 5],
                        Color::RGB(204, 204, 0),
                    )
                    .unwrap();
                internal_canvas
                    .filled_polygon(
                        &vec![flagx, flagx, flagx + 25],
                        &vec![flagy2, flagy2 - 10, flagy2 - 5],
                        Color::RGB(204, 204, 0),
                    )
                    .unwrap();
            },
            ScreenGuiTransformations::default(),
        );

        screen.render(mode)
    }

    pub fn new(render_mode: RenderMode, goal_velocity: Option<f64>) -> Self {
        let (mut rng, _) = rand_random(None);

        let min_position = OrderedFloat(-1.2);
        let max_position = OrderedFloat(0.6);
        let max_speed = OrderedFloat(0.07);
        let goal_position = OrderedFloat(0.5);
        let goal_velocity = OrderedFloat(goal_velocity.unwrap_or(0.));

        let force = OrderedFloat(0.001);
        let gravity = OrderedFloat(0.0025);

        let low = MountainCarObservation::new(min_position, -max_speed);
        let high = MountainCarObservation::new(max_position, max_speed);

        let renderer = Renderer::new(render_mode, None, None);

        let state = MountainCarObservation::sample_between(&mut rng, None);

        let metadata = Metadata::default();
        let screen = Screen::new(400, 600, "Mountain Car", metadata.render_fps, render_mode);

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

            render_mode,
            renderer,

            action_space,
            observation_space,

            state,
            rand_random: rng,

            screen,

            metadata,
        }
    }
}

impl Env for MountainCarEnv {
    type Action = usize;
    type Observation = MountainCarObservation;
    type Info = ();
    type ActionSpace = Discrete;
    type ObservationSpace = spaces::BoxR<Self::Observation>;
    type ResetInfo = ();

    fn step(&mut self, action: Self::Action) -> ActionReward<Self::Observation, Self::Info> {
        assert!(
            self.action_space.contains(action),
            "{} (usize) invalid",
            action
        );

        let mut position = self.state.position;
        let mut velocity = self.state.velocity;

        velocity += OrderedFloat((action as f64) - 1.) * self.force
            + (OrderedFloat(3.) * position).cos() * (-self.gravity);
        velocity = custom::clip(velocity, -self.max_speed, self.max_speed);

        position += velocity;
        position = custom::clip(position, self.min_position, self.max_position);

        if position == self.min_position && velocity < OrderedFloat(0.) {
            velocity = OrderedFloat(0.);
        }

        let done: bool = position >= self.goal_position && velocity >= self.goal_velocity;
        let reward: O64 = OrderedFloat(-1.0);

        self.state = MountainCarObservation { position, velocity };
        self.render(self.render_mode);

        ActionReward {
            observation: self.state,
            reward,
            done,
            truncated: false,
            info: None,
        }
    }

    fn render(&mut self, mode: RenderMode) -> Renders {
        let max_position = self.max_position;
        let min_position = self.min_position;
        let goal_position = self.goal_position;
        let state = self.state;
        let screen = &mut self.screen;
        let metadata = &self.metadata;

        let render_fn = &mut |mode| {
            Self::render(
                mode,
                max_position,
                min_position,
                goal_position,
                state,
                screen,
                metadata,
            )
        };

        if self.render_mode == RenderMode::None {
            self.renderer.get_renders(render_fn)
        } else {
            render_fn(mode)
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

        self.renderer.reset();

        let max_position = self.max_position;
        let min_position = self.min_position;
        let goal_position = self.goal_position;
        let state = self.state;
        let screen = &mut self.screen;
        let metadata = &self.metadata;

        self.renderer.render_step(&mut |mode| {
            Self::render(
                mode,
                max_position,
                min_position,
                goal_position,
                state,
                screen,
                metadata,
            )
        });

        if return_info {
            (self.state, Some(()))
        } else {
            (self.state, None)
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

    fn close(&mut self) {
        self.screen.gui.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_run() {
        pretty_env_logger::try_init().unwrap_or(());
        let mut mc = MountainCarEnv::new(RenderMode::Human, None);
        let _state = mc.reset(None, false, None);

        let mut end: bool = false;
        let mut episode_length = 0;
        while !end {
            if episode_length > 200 {
                break;
            }
            let action = (&mut thread_rng()).gen_range(0..3);
            let ActionReward { done, .. } = mc.step(action);
            episode_length += 1;
            end = done;
            println!("episode_length: {}", episode_length);
        }

        mc.close();

        for _ in 0..200 {
            let action = (&mut thread_rng()).gen_range(0..3);
            mc.step(action);
            episode_length += 1;
            println!("episode_length: {}", episode_length);
        }
    }

    #[test]
    fn test_clone() {
        let mc = MountainCarEnv::new(RenderMode::None, None);
        let _mc_cloned = mc.clone();

        let mc2 = MountainCarEnv::new(RenderMode::Human, None);
        let _mc2_cloned = mc2.clone();
    }
}
