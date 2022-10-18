use log::warn;
use nalgebra as na;
use ordered_float::{impl_rand::UniformOrdered, Float};
use sdl2::{
    gfx::primitives::DrawRenderer,
    pixels::{self, Color},
};
use std::{f64::consts::PI, ops::Neg};

use derive_new::new;
use ordered_float::OrderedFloat;
use rand::{
    distributions::{
        uniform::{SampleUniform, UniformSampler},
        Uniform,
    },
    prelude::Distribution,
    Rng,
};

use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    core::{ActionReward, Env, EnvProperties},
    spaces::{BoxR, Discrete, Space},
    utils::{
        custom::{Metadata, Sample, Screen, ScreenGuiTransformations, O64},
        renderer::{RenderMode, Renderer, Renders},
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
    /// The type of renders produced.
    pub render_mode: RenderMode,
    /// The current state of the environment.
    pub state: CartPoleObservation,
    /// Additional pieces of information provided by the environment.
    pub metadata: Metadata<Self>,
    /// The gravity constant applied to the environment..
    pub gravity: O64,
    /// The mass of the cart.
    pub masscart: O64,
    /// The mass of the pole.
    pub masspole: O64,
    /// Half the length of the pole.
    pub length: O64,
    /// The default force applied to the pole.
    pub force_mag: O64,
    /// The number of seconds between state updates.
    pub tau: O64,
    /// The type of integration done on the differential equations found in the paper.
    pub kinematics_integrator: KinematicsIntegrator,
    /// The angle that the pole can lean to before an episode is considered terminated.
    pub theta_threshold_radians: O64,
    /// The x value that the cart can be at before an episode is considered terminated.
    pub x_threshold: O64,
    /// The number of steps taken after the episode was terminated.
    pub steps_beyond_terminated: Option<usize>,
    renderer: Renderer,
    screen: Screen,
    #[serde(skip_serializing)]
    rand_random: Pcg64,
}

impl CartPoleEnv {
    /// Creates a cart pole environment using defaults from the paper.
    pub fn new(render_mode: RenderMode) -> Self {
        let (mut rand_random, _) = rand_random(None);

        let gravity = OrderedFloat(9.8);
        let masscart = OrderedFloat(1.0);
        let masspole = OrderedFloat(0.1);
        let length = OrderedFloat(0.5);
        let force_mag = OrderedFloat(10.0);
        let tau = OrderedFloat(0.02);
        let kinematics_integrator = KinematicsIntegrator::Euler;

        let theta_threshold_radians = OrderedFloat(12. * 2. * PI / 360.);
        let x_threshold = OrderedFloat(2.4);

        let high = CartPoleObservation::new(
            x_threshold * 2.,
            OrderedFloat(f64::INFINITY),
            theta_threshold_radians * 2.,
            OrderedFloat(f64::INFINITY),
        );

        let action_space = Discrete(2);
        let observation_space = BoxR::new(-high, high);

        let renderer = Renderer::new(render_mode, None, None);

        let metadata = Metadata::default();
        let screen = Screen::new(400, 600, "Cart Pole", metadata.render_fps, render_mode);

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
            render_mode,
            renderer,
            screen,
            state,
            metadata,
            rand_random,
            steps_beyond_terminated,
        }
    }

    fn total_mass(&self) -> O64 {
        self.masspole + self.masscart
    }

    fn polemass_length(&self) -> O64 {
        self.masspole + self.length
    }

    fn render(
        mode: RenderMode,
        screen: &mut Screen,
        metadata: &Metadata<Self>,
        x_threshold: O64,
        length: O64,
        state: CartPoleObservation,
    ) -> Renders {
        assert!(metadata.render_modes.contains(&mode));

        screen.load_gui();
        screen.consume_events();

        let screen_width = screen.screen_width();
        let world_width = x_threshold * 2.;
        let scale = OrderedFloat(screen_width as f64) / world_width;
        let polewidth: O64 = OrderedFloat(10.);
        let polelen = scale * 2. * length;
        let cartwidth = OrderedFloat(50.);
        let cartheight = OrderedFloat(30.);

        screen.draw_on_canvas(
            |canvas| {
                canvas.set_draw_color(pixels::Color::WHITE);
                canvas.clear();

                let (mut l, mut r, mut t, mut b) = (
                    -cartwidth / OrderedFloat(2f64),
                    cartwidth / OrderedFloat(2f64),
                    cartheight / OrderedFloat(2f64),
                    -cartheight / OrderedFloat(2f64),
                );

                let axleoffset = cartheight / OrderedFloat(4.0);
                let cartx = state.x * scale + OrderedFloat(screen_width as f64) / OrderedFloat(2.0);
                let carty = OrderedFloat(100.);
                let cart_coords = [(l, b), (l, t), (r, t), (r, b)]
                    .map(|(x, y)| (x + cartx, y + carty))
                    .map(|(x, y)| (x.floor().into_inner() as i16, y.floor().into_inner() as i16));

                let cart_coords_x = &cart_coords.map(|coord| coord.0);
                let cart_coords_y = &cart_coords.map(|coord| coord.1);

                canvas
                    .aa_polygon(cart_coords_x, cart_coords_y, pixels::Color::BLACK)
                    .unwrap();

                canvas
                    .filled_polygon(cart_coords_x, cart_coords_y, pixels::Color::BLACK)
                    .unwrap();

                (l, r, t, b) = (
                    -polewidth / OrderedFloat(2f64),
                    polewidth / OrderedFloat(2f64),
                    polelen - polewidth / OrderedFloat(2f64),
                    -polewidth / OrderedFloat(2f64),
                );

                let pole_coords = [(l, b), (l, t), (r, t), (r, b)].map(|(x, y)| {
                    let rotation_matrix = na::Rotation2::new(-state.theta.into_inner());
                    let point = na::Point2::new(x.into_inner(), y.into_inner());
                    let rotated_point = rotation_matrix * point;
                    (
                        rotated_point.x + cartx.into_inner(),
                        rotated_point.y + (carty + axleoffset).into_inner(),
                    )
                });

                let pole_coords_x = &pole_coords.map(|coord| coord.0 as i16);
                let pole_coords_y = &pole_coords.map(|coord| coord.1 as i16);

                canvas
                    .aa_polygon(pole_coords_x, pole_coords_y, Color::RGB(202, 152, 101))
                    .unwrap();
                canvas
                    .filled_polygon(pole_coords_x, pole_coords_y, Color::RGB(202, 152, 101))
                    .unwrap();

                canvas
                    .aa_circle(
                        cartx.floor().into_inner() as i16,
                        (carty + axleoffset).floor().into_inner() as i16,
                        (polewidth / OrderedFloat(2f64)).floor().into_inner() as i16,
                        Color::RGB(129, 132, 203),
                    )
                    .unwrap();
                canvas
                    .filled_circle(
                        cartx.floor().into_inner() as i16,
                        (carty + axleoffset).floor().into_inner() as i16,
                        (polewidth / OrderedFloat(2f64)).floor().into_inner() as i16,
                        Color::RGB(129, 132, 203),
                    )
                    .unwrap();

                canvas
                    .hline(
                        0,
                        screen_width as i16,
                        carty.floor().into_inner() as i16,
                        Color::BLACK,
                    )
                    .unwrap();
            },
            ScreenGuiTransformations::default(),
        );

        screen.render(mode)
    }
}

const CART_POLE_RENDER_MODES: &'static [RenderMode] = &[RenderMode::Human, RenderMode::RgbArray];

impl Default for Metadata<CartPoleEnv> {
    fn default() -> Self {
        Metadata::new(CART_POLE_RENDER_MODES, 50)
    }
}

/// The sampler responsible for generating an observation using uniform probability.
pub struct UniformCartPoleObservation {
    x_sampler: UniformOrdered<f64>,
    x_dot_sampler: UniformOrdered<f64>,
    theta_sampler: UniformOrdered<f64>,
    theta_dot_sampler: UniformOrdered<f64>,
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
            x_sampler: UniformOrdered::new(low.borrow().x, high.borrow().x),
            x_dot_sampler: UniformOrdered::new(low.borrow().x_dot, high.borrow().x_dot),
            theta_sampler: UniformOrdered::new(low.borrow().theta, high.borrow().theta),
            theta_dot_sampler: UniformOrdered::new(low.borrow().theta_dot, high.borrow().theta_dot),
        }
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
    {
        UniformCartPoleObservation {
            x_sampler: UniformOrdered::new_inclusive(low.borrow().x, high.borrow().x),
            x_dot_sampler: UniformOrdered::new_inclusive(low.borrow().x_dot, high.borrow().x_dot),
            theta_sampler: UniformOrdered::new_inclusive(low.borrow().theta, high.borrow().theta),
            theta_dot_sampler: UniformOrdered::new_inclusive(
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
#[derive(new, Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct CartPoleObservation {
    x: O64,
    x_dot: O64,
    theta: O64,
    theta_dot: O64,
}

impl From<CartPoleObservation> for Vec<f64> {
    fn from(observation: CartPoleObservation) -> Self {
        (vec![
            observation.x,
            observation.x_dot,
            observation.theta,
            observation.theta_dot,
        ])
        .iter()
        .map(|v| v.into_inner())
        .collect()
    }
}

impl Sample for CartPoleObservation {
    fn sample_between<R: Rng>(rng: &mut R, bounds: Option<BoxR<Self>>) -> Self {
        let BoxR { low, high } = bounds.unwrap_or({
            let observation_bound = CartPoleObservation::new(
                OrderedFloat(0.5),
                OrderedFloat(0.5),
                OrderedFloat(0.5),
                OrderedFloat(0.5),
            );
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
    type Action = usize;

    type Observation = CartPoleObservation;

    type Info = ();

    type ResetInfo = ();

    fn step(
        &mut self,
        action: Self::Action,
    ) -> crate::core::ActionReward<Self::Observation, Self::Info> {
        assert!(
            self.action_space.contains(action),
            "{} usize invalid",
            action
        );

        if self.steps_beyond_terminated.is_some() {}

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

        let temp = (force + self.polemass_length() * theta_dot.powf(OrderedFloat(2.)) * sintheta)
            / self.total_mass();
        let thetaacc = (self.gravity * sintheta - costheta * temp)
            / (self.length
                * (OrderedFloat(4.0 / 3.0)
                    - self.masspole * costheta.powf(OrderedFloat(2.)) / self.total_mass()));
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
            OrderedFloat(1.0)
        } else if self.steps_beyond_terminated.is_none() {
            self.steps_beyond_terminated = Some(0);
            OrderedFloat(1.0)
        } else {
            warn!("Calling step after termination may result in undefined behaviour. Consider reseting.");
            self.steps_beyond_terminated = self.steps_beyond_terminated.map(|step| step + 1);
            OrderedFloat(0.)
        };

        let screen = &mut self.screen;
        let metadata = &self.metadata;
        let x_threshold = self.x_threshold;
        let length = self.length;
        let state = self.state;

        self.renderer.render_step(&mut |mode| {
            Self::render(mode, screen, metadata, x_threshold, length, state)
        });

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

        self.renderer.reset();

        let screen = &mut self.screen;
        let metadata = &self.metadata;
        let x_threshold = self.x_threshold;
        let length = self.length;
        let state = self.state;

        self.steps_beyond_terminated = None;

        self.renderer.reset();
        self.renderer.render_step(&mut |mode| {
            Self::render(mode, screen, metadata, x_threshold, length, state)
        });

        if return_info {
            (self.state, Some(()))
        } else {
            (self.state, None)
        }
    }

    fn render(&mut self, mode: RenderMode) -> crate::utils::renderer::Renders {
        let screen = &mut self.screen;
        let metadata = &self.metadata;
        let x_threshold = self.x_threshold;
        let length = self.length;
        let state = self.state;

        let render_fn =
            &mut |mode| Self::render(mode, screen, metadata, x_threshold, length, state);
        if self.render_mode != RenderMode::None {
            self.renderer.get_renders(render_fn)
        } else {
            render_fn(mode)
        }
    }

    fn close(&mut self) {
        self.screen.close();
    }
}

impl EnvProperties for CartPoleEnv {
    type ActionSpace = Discrete;

    type ObservationSpace = BoxR<CartPoleObservation>;

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
