#![deny(missing_docs, missing_crate_level_docs)]

//! The gym-rs crate is a pure rust implementation of OpenAI's Gym

#[macro_use]
extern crate log;

mod action_type;
mod cart_pole;
mod gym_env;
mod mountain_car;
mod pendulum;
mod utils;
mod viewer;

pub use action_type::ActionType;
pub use cart_pole::CartPoleEnv;
pub use gym_env::GymEnv;
pub use mountain_car::MountainCarEnv;
pub use pendulum::PendulumEnv;
pub use utils::scale;
pub use viewer::Viewer;
