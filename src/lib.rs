#![deny(missing_docs, missing_crate_level_docs)]

//! The gym-rs crate is a pure rust implementation of OpenAI's Gym

#[macro_use]
extern crate log;

mod cart_pole;
pub mod core;
pub mod envs;
mod gif_render;
mod pendulum;
pub mod spaces;
mod utils;

pub use cart_pole::CartPoleEnv;
pub use gif_render::GifRender;
pub use pendulum::PendulumEnv;
pub use utils::scale;
