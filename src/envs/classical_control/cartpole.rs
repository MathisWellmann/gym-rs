use crate::utils::definitions::O64;

pub struct CartPole {
    pub gravity: O64,
    pub masscart: O64,
    pub masspole: O64,
    pub length: O64,
    pub force_mag: O64,
    pub tau: O64,
    pub kinematics_integrator: KinematicsIntegrator,
    pub theta_threshold_radians: O64,
}

pub enum KinematicsIntegrator {
    Euler,
    Other,
}
