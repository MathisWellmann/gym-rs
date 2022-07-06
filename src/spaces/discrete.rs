use serde::Serialize;

/// TODO
#[derive(Debug, Serialize)]
pub struct Discrete(pub usize);

impl Discrete {
    /// TODO
    pub fn contains(&self, value: usize) -> bool {
        value < self.0
    }
}
