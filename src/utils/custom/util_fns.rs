/// Clips a value between the left and right bounds.
pub fn clip<T: PartialEq + PartialOrd>(value: T, left_bound: T, right_bound: T) -> T {
    if left_bound <= value && value <= right_bound {
        value
    } else if value > right_bound {
        right_bound
    } else {
        left_bound
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::custom::util_fns::clip;

    #[test]
    fn given_bounds_when_value_is_beyond_upper_bound_then_upper_bound_returned() {
        let received_val = clip(2, 0, 1);
        assert_eq!(received_val, 1);
    }

    #[test]
    fn given_bounds_when_value_is_less_than_lower_bound_then_lower_bound_returned() {
        let received_val = clip(-1, 0, 1);
        assert_eq!(received_val, 0);
    }

    #[test]
    fn given_bounds_when_value_is_between_boudns_then_value_returned() {
        let recieved_val = clip(1, -1, 2);
        assert_eq!(recieved_val, 1);
    }
}
