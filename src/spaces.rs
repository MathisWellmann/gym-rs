pub struct Discrete(u8);

impl Discrete {
    pub fn contains(value: u8) {
        value < self.0
    }
}

pub struct Box<T>([Float<T>; 2], [Float<T>; 2]);
