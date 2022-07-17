use std::{f64::consts::PI, ops::Neg};

use derive_new::new;
use ordered_float::OrderedFloat;
use rand::{
    distributions::{
        uniform::{SampleUniform, UniformSampler},
        Uniform,
    },
    prelude::Distribution,
};

use ordered_float::impl_rand::UniformOrdered;
use rand_pcg::Pcg64;
use sdl2::{pixels::PixelFormatEnum, render::Texture};
use serde::Serialize;

use crate::{
    core::Env,
    spaces::{BoxR, Discrete},
    utils::{
        custom::{Metadata, Sample, Screen, O64},
        renderer::{RenderMode, Renderer, Renders},
        seeding::{self, rand_random},
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct CartPoleEnv<'a> {
    pub gravity: O64,
    pub masscart: O64,
    pub masspole: O64,
    pub length: O64,
    pub force_mag: O64,
    pub tau: O64,
    pub kinematics_integrator: KinematicsIntegrator,
    pub theta_threshold_radians: O64,
    pub x_threshold: O64,
    pub action_space: Discrete,
    pub observation_space: BoxR<CartPoleObservation>,
    pub render_mode: RenderMode,
    pub renderer: Renderer<'a>,
    pub screen: Screen,
    pub state: CartPoleObservation,
    pub metadata: Metadata<Self>,
    #[serde(skip_serializing)]
    rand_random: Pcg64,
}

impl<'a> CartPoleEnv<'a> {
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
        let screen = Screen::new(600, 400, "Cart Pole", metadata.render_fps, render_mode);

        let state = CartPoleObservation::sample_between(&mut rand_random, None);

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
        }
    }

    pub fn total_mass(&self) -> O64 {
        self.masspole + self.masscart
    }

    pub fn polemass_length(&self) -> O64 {
        self.masspole + self.length
    }

    pub fn render(
        mode: RenderMode,
        screen: &mut Screen,
        metadata: Metadata<Self>,
        x_threshold: O64,
        length: O64,
    ) -> Renders {
        assert!(metadata.render_modes.contains(&mode));

        screen.load_gui();
        screen.consume_events();

        let gui_manager = screen.gui.as_mut().expect("GUI not found.");
        let canvas = &mut gui_manager.canvas;
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_target(PixelFormatEnum::RGB24, screen.width, screen.height)
            .expect("Texture was unable to be created");

        let world_width = x_threshold * 2.;
        let scale = OrderedFloat(screen.width as f64) / world_width;
        let polewidth = OrderedFloat(10.);
        let polelen = scale * 2. * length;
        let cartwidth = OrderedFloat(50.);
        let cartheight = OrderedFloat(30.);

        todo!()
    }
}

const CART_POLE_RENDER_MODES: &'static [RenderMode] = &[
    RenderMode::Human,
    RenderMode::RgbArray,
    RenderMode::SingleRgbArray,
    RenderMode::DepthArray,
    RenderMode::SingleDepthArray,
];

impl<'a> Default for Metadata<CartPoleEnv<'a>> {
    fn default() -> Self {
        Metadata::new(CART_POLE_RENDER_MODES, 20)
    }
}

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
    fn sample_between(rng: &mut Pcg64, bounds: Option<BoxR<Self>>) -> Self {
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

#[derive(Clone, Debug, Serialize)]
pub enum KinematicsIntegrator {
    Euler,
    Other,
}

impl<'a> Env for CartPoleEnv<'a> {
    type Action = usize;

    type Observation = CartPoleObservation;

    type Info = ();

    type ActionSpace = Discrete;

    type ObservationSpace = BoxR<CartPoleObservation>;

    type ResetInfo = ();

    fn step(
        &mut self,
        action: Self::Action,
    ) -> crate::core::ActionReward<Self::Observation, Self::Info> {
        todo!()
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

        todo!()
    }

    fn render(&mut self, mode: RenderMode) -> crate::utils::renderer::Renders {
        todo!()
    }

    fn close(&mut self) {
        self.screen.gui.take();
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
}
