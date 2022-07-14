use serde::Serialize;

use super::Space;

#[derive(Debug, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct Discrete(pub usize);

impl Space<usize> for Discrete {
    fn contains(&self, value: usize) -> bool {
        match *self {
            Discrete(upper_bound) => value < upper_bound,
        }
    }
}
