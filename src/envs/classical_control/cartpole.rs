use std::os::linux::fs::MetadataExt;

use crate::{
    spaces::{BoxR, Discrete},
    utils::{
        custom::{Metadata, Screen, O64},
        renderer::{RenderMode, Renderer},
    },
};

pub struct CartPole<'a> {
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
    pub observation_space: BoxR<O64>,
    pub render_mode: RenderMode,
    pub renderer: Renderer<'a>,
    pub screen_width: u32,
    pub screen_height: u32,
    pub screen: Option<Screen>,
    pub state: CartPoleObservation,
}

impl<'a> CartPole<'a> {
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

impl<'a> Default for Metadata<CartPole<'a>> {
    fn default() -> Self {
        Metadata::new(CART_POLE_RENDER_MODES, 20)
    }
}

pub struct CartPoleObservation {}

pub enum KinematicsIntegrator {
    Euler,
    Other,
}
