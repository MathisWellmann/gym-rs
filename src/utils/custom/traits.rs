use rand::{distributions::uniform::SampleUniform, Rng};

use crate::spaces::BoxR;

/// Defines a set of operations to sample an observation for an environment.
pub trait Sample: SampleUniform {
    /// Retrieves a randomly generated observation between the given bounds.
    fn sample_between<R: Rng>(rng: &mut R, bounds: Option<BoxR<Self>>) -> Self;
}
