use std::marker::PhantomData;

use derive_new::new;
use serde::Serialize;

use crate::utils::renderer::RenderMode;

/// Defines a set of common properties used to describe the environment further.
///
/// Can be dynamically altered and outputted during a state output to describe
/// the contents of the state further.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Ord, PartialOrd, Copy, new)]
pub struct Metadata<T> {
    /// Defines the render modes supported by the environment.
    pub render_modes: &'static [RenderMode],
    /// Defines the fps used by the internal renderer.
    pub render_fps: u32,
    marker: PhantomData<T>,
}
