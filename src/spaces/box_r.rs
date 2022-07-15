use derive_new::new;
use serde::Serialize;

#[derive(Debug, Serialize, new, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct BoxR<T> {
    pub lower_bound: T,
    pub upper_bound: T,
}
