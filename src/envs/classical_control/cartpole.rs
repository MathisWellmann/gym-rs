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

use crate::{
    spaces::{BoxR, Discrete},
    utils::{
        custom::{DefaultSeed, Metadata, Screen, O64},
        renderer::{RenderMode, Renderer},
        seeding::rand_random,
    },
};

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
    pub screen_width: u32,
    pub screen_height: u32,
    pub screen: Option<Screen>,
    pub state: CartPoleObservation,
}

impl<'a> CartPoleEnv<'a> {
    pub fn new(render_mode: RenderMode) -> Self {
        let (mut rng, _) = rand_random(None);

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

        let screen_width = 600;
        let screen_height = 400;
        let screen = None;

        let state = CartPoleObservation::default(&mut rng);

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
            screen_width,
            screen_height,
            screen,
            state,
        }
    }

    pub fn total_mass(&self) -> O64 {
        self.masspole + self.masscart
    }

    pub fn polemass_length(&self) -> O64 {
        self.masspole + self.length
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

#[derive(new, Debug, Clone, Copy)]
pub struct CartPoleObservation {
    x: O64,
    x_dot: O64,
    theta: O64,
    theta_dot: O64,
}

impl DefaultSeed for CartPoleObservation {
    fn default(rng: &mut Pcg64) -> Self {
        let bound = CartPoleObservation {
            x: OrderedFloat(0.5),
            x_dot: OrderedFloat(0.5),
            theta: OrderedFloat(0.5),
            theta_dot: OrderedFloat(0.5),
        };
        Uniform::new(-bound, bound).sample(rng)
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

pub enum KinematicsIntegrator {
    Euler,
    Other,
}
