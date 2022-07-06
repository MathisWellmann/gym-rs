use derive_new::new;
use serde::Serialize;

#[derive(Debug, Serialize, new)]
pub struct Box<T> {
    pub lower_bound: T,
    pub upper_bound: T,
}
