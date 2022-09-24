#![deny(missing_docs)]

extern crate log;

/// Contains user-facing interfaces.
pub mod core;
/// Holds implementations of various environments.
pub mod envs;
/// Holds structures describing collections of values.
pub mod spaces;
/// Functions, structures and traits designed to reduce complex interactions.
pub mod utils;
