use crate::{
    spaces::{BoxR, Discrete},
    utils::{
        custom::{Screen, O64},
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

pub struct CartPoleObservation {}

pub enum KinematicsIntegrator {
    Euler,
    Other,
}
