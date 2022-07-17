use derive_new::new;
use serde::Serialize;

#[derive(Debug, Serialize, new, Clone)]
pub struct BoxR<T> {
    pub low: T,
    pub high: T,
}
