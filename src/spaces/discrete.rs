#[derive(Debug)]
pub struct Discrete(pub u8);

impl Discrete {
    pub fn contains(&self, value: u8) -> bool {
        value < self.0
    }
}
