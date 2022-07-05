#[derive(Debug)]
pub struct Discrete(pub usize);

impl Discrete {
    pub fn contains(&self, value: usize) -> bool {
        value < self.0
    }
}
