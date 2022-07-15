use crate::utils::definitions::O64;

struct CartPole {
    gravity: O64,
    masscart: O64,
    masspole: O64,
    length: O64,
    force_mag: O64,
    tau: O64,
    kinematics_integrator: KinematicsIntegrator,
    theta_threshold_radians: O64,
}

enum KinematicsIntegrator {
    Euler,
    Other,
}
