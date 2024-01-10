use serde::Serialize;

use super::Space;

/// Defines a set of discrete integers starting at 0.
///
/// The value held by this structure defines the largest inclusive value that
/// exists within the derived set.  
///
/// TODO: Update to support negative values.
#[derive(Debug, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct Discrete(pub usize);

impl Space<usize> for Discrete {
    fn contains(&self, value: usize) -> bool {
        match *self {
            Discrete(upper_bound) => value < upper_bound,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Discrete;
    use crate::spaces::Space;

    #[test]
    fn given_value_greater_or_eq_than_upper_bound_when_contains_called_then_returns_false() {
        let obj = Discrete(3);

        assert!(!obj.contains(3));
        assert!(!obj.contains(4));
    }

    #[test]
    fn given_value_less_than_upper_bound_when_contains_then_returns_true() {
        let obj = Discrete(3);

        assert!(obj.contains(1));
        assert!(obj.contains(2));
    }
}
