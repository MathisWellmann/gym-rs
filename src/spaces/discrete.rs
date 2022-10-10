use serde::Serialize;

use super::Space;

#[derive(Debug, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
/// Defines a set of discrete natural numbers
///
/// The value held by this structure defines the largest inclusive value that
/// exists within the derived set.  
pub struct Discrete(pub usize);

impl Space<usize> for Discrete {
    fn contains(&self, value: usize) -> bool {
        match *self {
            Discrete(upper_bound) => value < upper_bound,
        }
    }
}
