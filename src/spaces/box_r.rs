use derive_new::new;
use serde::Serialize;

/// Defines a subspace created between two points.
#[derive(Debug, Serialize, new, Clone)]
pub struct BoxR<T> {
    /// Defines the lower bound of the subspace where values less than what
    /// is inputted cannot exist.
    pub low: T,
    /// Defines the upper bound of the subspace where values greater than what is
    /// inputted cannot exist.
    pub high: T,
}
