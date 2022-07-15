use derive_new::new;

use crate::{spaces::BoxR, utils::definitions::O64};

#[derive(new, PartialEq, Eq, PartialOrd, Debug, Clone, Ord)]
pub struct MaybeParseResetBoundsOptions {
    low: Option<O64>,
    high: Option<O64>,
}

pub fn maybe_parse_reset_bounds(
    options: Option<MaybeParseResetBoundsOptions>,
    default_low: O64,
    default_high: O64,
) -> BoxR<O64> {
    if options.is_none() {
        BoxR::new(default_low, default_high)
    } else {
        let low = options.as_ref().and_then(|o| o.low).unwrap_or(default_low);
        let high = options
            .as_ref()
            .and_then(|o| o.high)
            .unwrap_or(default_high);

        assert!(high > low, "options.low must be less than options.high");

        BoxR::new(low, high)
    }
}
