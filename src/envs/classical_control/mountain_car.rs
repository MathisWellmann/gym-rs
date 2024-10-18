use std::{fmt::Debug, iter::zip};

use derivative::Derivative;
use derive_new::new;
use na::{Point2, Rotation2};
use nalgebra as na;
use num_traits::Float;
use ordered_float::{OrderedFloat, UniformOrdered};
use rand::{
    distributions::{
        uniform::{SampleUniform, UniformSampler},
        Uniform,
    },
    prelude::Distribution,
    Rng,
};
use rand_pcg::Pcg64;
use sdl2::{gfx::primitives::DrawRenderer, pixels::Color, rect::Point};
use serde::Serialize;

use crate::{
    core::{ActionReward, Env, EnvProperties},
    spaces::{self, BoxR, Discrete, Space},
    utils::{
        custom::{
            screen::{Screen, ScreenGuiTransformations},
            structs::Metadata,
            traits::Sample,
            types::O64,
            util_fns::clip,
        },
        renderer::{RenderMode, Renderer, Renders},
        seeding::rand_random,
    },
};

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

    /// The type of renders produced.
    pub render_mode: RenderMode,

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
    screen: Screen,
    renderer: Renderer,
}

impl Clone for MountainCarEnv {
    fn clone(&self) -> Self {
        Self {
            min_position: self.min_position,
            max_position: self.max_position,
            max_speed: self.max_speed,
            goal_position: self.goal_position,
            goal_velocity: self.goal_velocity,
            force: self.force,
            gravity: self.gravity,
            render_mode: self.render_mode,
            renderer: self.renderer.clone(),
            screen: self.screen.clone(),
            action_space: self.action_space.clone(),
            observation_space: self.observation_space.clone(),
            state: self.state,
            rand_random: self.rand_random.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

const MOUNTAIN_CAR_RENDER_MODES: &[RenderMode] = &[
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
    /// The position the car exists on the mountain.
    pub position: O64,
    /// The velocity the car is travelling at.
    pub velocity: O64,
}

/// The structure reponsible for uniformly sampling a mountain car observation.
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
    fn height(xs: &[O64]) -> Vec<O64> {
        Vec::from_iter(
            xs.iter()
                .map(|value| OrderedFloat((3. * value.into_inner()).sin() * 0.45 + 0.55)),
        )
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
        let scale = OrderedFloat(screen.screen_width() as f64) / world_width;
        let carwidth = 40;
        let carheight = 20;

        screen.draw_on_canvas(
            |internal_canvas| {
                internal_canvas.set_draw_color(Color::WHITE);
                internal_canvas.clear();

                let pos = state.position;

                let xs = Vec::from_iter(
                    (0..100)
                        .map(|index| (((max_position - min_position) / 100.) * index as f64))
                        .map(|value| value + min_position),
                );

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
                    let new_y =
                        OrderedFloat(y) + clearance + Self::height(&[pos]).pop().unwrap() * scale;

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
                    let point = Point2::new(x, y);
                    let desired_angle = ((OrderedFloat(3.) * pos).cos()).into_inner();
                    let rotation_matrix = Rotation2::new(desired_angle);
                    let rotated_point = rotation_matrix.transform_point(&point);

                    let (x, y) = (OrderedFloat(rotated_point.x), OrderedFloat(rotated_point.y));

                    let (wheel_x, wheel_y) = (
                        (x + (pos - min_position) * scale).floor().into_inner() as i16,
                        (y + clearance + Self::height(&[pos]).pop().unwrap() * scale)
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
                let flagy1 = (Self::height(&[goal_position]).pop().unwrap() * scale)
                    .floor()
                    .into_inner() as i16;
                let flagy2 = flagy1 + 50;
                internal_canvas
                    .vline(flagx, flagy1, flagy2, Color::BLACK)
                    .unwrap();

                internal_canvas
                    .aa_polygon(
                        &[flagx, flagx, flagx + 25],
                        &[flagy2, flagy2 - 10, flagy2 - 5],
                        Color::RGB(204, 204, 0),
                    )
                    .unwrap();
                internal_canvas
                    .filled_polygon(
                        &[flagx, flagx, flagx + 25],
                        &[flagy2, flagy2 - 10, flagy2 - 5],
                        Color::RGB(204, 204, 0),
                    )
                    .unwrap();
            },
            ScreenGuiTransformations::default(),
        );

        screen.render(mode)
    }

    /// Generates an instance of the mountain car environment using the defaults provided in the
    /// paper.
    pub fn new(render_mode: RenderMode) -> Self {
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

    fn close(&mut self) {
        self.screen.close();
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
